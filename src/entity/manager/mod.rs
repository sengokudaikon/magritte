pub(crate) mod cache;
pub mod macros;
pub(crate) mod registry;
pub(crate) mod unsafe_em;
use crate::entity::manager::cache::EntityCache;
use crate::entity::manager::unsafe_em::TypeErasedState;
use crate::{
    ColumnTrait, EdgeTrait, HasColumns, HasRelations, NamedType, RecordType, RelationTrait,
    TableTrait,
};
use anyhow::{anyhow, Error, Result};
pub use macros::EntityFlusherRegistration;
pub use macros::EntityStateFlusher;
use magritte_query::database::{QueryType, SurrealDB};
use magritte_query::{HasId, Query, SurrealId};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::sync::Arc;
use futures_locks::RwLock;
use surrealdb::Response;
use tracing::error;

/// Type-safe container for entity state tracking
#[derive(Default)]
pub struct EntityState<T: TableTrait> {
    pub new: Vec<T>,
    pub managed: HashMap<String, T>,
    pub removed: Vec<T>,
    pub dirty: HashSet<String>,
}

impl<T: TableTrait> EntityState<T> {
    pub fn new() -> Self {
        Self {
            new: Vec::new(),
            managed: HashMap::new(),
            removed: Vec::new(),
            dirty: HashSet::new(),
        }
    }
}

pub struct UnitOfWork {
    states: HashMap<TypeId, TypeErasedState>,
}

impl Default for UnitOfWork {
    fn default() -> Self {
        Self::new()
    }
}
impl UnitOfWork {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    fn get_or_create_state<T: TableTrait + 'static>(&mut self) -> &mut EntityState<T> {
        let type_id = TypeId::of::<T>();
        self.states
            .entry(type_id)
            .or_insert_with(|| TypeErasedState::new(EntityState::<T>::new()));
        // Safety: We just checked the type_id matches
        unsafe { self.states.get_mut(&type_id).unwrap().as_mut().unwrap() }
    }

    pub fn mark_new<T>(&mut self, entity: T)
    where
        T: TableTrait + 'static,
    {
        let state = self.get_or_create_state::<T>();
        state.new.push(entity);
    }

    pub fn mark_managed<T>(&mut self, entity: T)
    where
        T: TableTrait + HasId + 'static,
    {
        let state = self.get_or_create_state::<T>();
        let id = entity.id().to_string();
        state.managed.insert(id, entity);
    }

    pub fn mark_removed<T>(&mut self, entity: T)
    where
        T: TableTrait + 'static,
    {
        let state = self.get_or_create_state::<T>();
        state.removed.push(entity);
    }

    pub fn mark_dirty<T>(&mut self, id: String)
    where
        T: TableTrait + 'static,
    {
        let state = self.get_or_create_state::<T>();
        state.dirty.insert(id);
    }

    pub fn get_state<T: TableTrait + 'static>(&self) -> Option<&EntityState<T>> {
        self.states
            .get(&TypeId::of::<T>())
            .and_then(|state| unsafe { state.as_ref() })
    }

    pub fn get_state_mut<T: TableTrait + 'static>(&mut self) -> Option<&mut EntityState<T>> {
        self.states
            .get_mut(&TypeId::of::<T>())
            .and_then(|state| unsafe { state.as_mut() })
    }
}

#[derive(Debug, Clone)]
pub struct LoadingConfig {
    pub batch_size: usize,
    pub max_depth: usize,
    pub timeout: std::time::Duration,
}

impl Default for LoadingConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            max_depth: 3,
            timeout: std::time::Duration::from_secs(30),
        }
    }
}

/// Our EntityManager
#[derive(Clone)]
pub struct EntityManager {
    db: SurrealDB,
    cache: Arc<EntityCache>,
    unit_of_work: Arc<RwLock<UnitOfWork>>,
    loading_config: LoadingConfig,
}

impl EntityManager {
    pub fn new(db: SurrealDB, cache: EntityCache, config: Option<LoadingConfig>) -> Self {
        // Ensure all registrations are collected
        let _ = registry::get_registered_entities();

        Self {
            db,
            cache: Arc::new(cache),
            unit_of_work: Arc::new(RwLock::new(UnitOfWork::new())),
            loading_config: config.unwrap_or_default(),
        }
    }

    pub fn db(&self) -> SurrealDB {
        self.db.clone()
    }

    fn clone_self(&self) -> Arc<Self> {
        Arc::new(Self {
            db: self.db.clone(),
            cache: self.cache.clone(),
            unit_of_work: Arc::new(RwLock::new(UnitOfWork::new())),
            loading_config: self.loading_config.clone(),
        })
    }

    /// Finds an entity by ID, caching it if not present.
    pub async fn find<T>(&self, id: &str) -> Result<Option<T>>
    where
        T: TableTrait + HasId + HasColumns + HasRelations + Send + Sync + 'static,
    {
        // Check cache first
        if let Some(entity) = self.cache.get_entity::<T>(T::table_name(), id).await? {
            return Ok(Some(entity));
        }

        // Just select the entity:
        let mut results: Vec<T> = Query::select::<T>()
            .where_id(SurrealId::<T>::from(id))
            .execute(&self.db())
            .await
            .map_err(anyhow::Error::from)?;
        if let Some(entity) = results.pop() {
            self.cache
                .cache_entity(T::table_name(), id, &entity)
                .await?;
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }

    /// Load a given relation. Returns (Source, Vec<Target>) for potentially multiple targets.
    pub async fn load_relation<R>(
        &self,
        source_id: &str,
        relation: R,
    ) -> Result<(R::Source, Vec<R::Target>)>
    where
        R: RelationTrait + 'static + Send + Sync,
        R::Source: TableTrait + HasId + Send + Sync + 'static,
        R::Target: TableTrait + HasId + Send + Sync + 'static,
    {
        // Try cache first:
        let def = relation.def_owned();
        if let Some(cached_ids) = self.cache.get_related_ids(&def).await? {
            let source = self
                .cache
                .get_entity::<R::Source>(R::Source::table_name(), source_id)
                .await?
                .ok_or_else(|| anyhow!("Source not cached"))?;

            let mut targets = Vec::new();
            let mut all_cached = true;
            for tid in &cached_ids {
                if let Some(t) = self
                    .cache
                    .get_entity::<R::Target>(R::Target::table_name(), tid)
                    .await?
                {
                    targets.push(t);
                } else {
                    all_cached = false;
                    break;
                }
            }

            if all_cached {
                return Ok((source, targets));
            }
            // Otherwise, fall through to fresh load
        }

        self.load_relation_fresh::<R>(source_id).await
    }

    async fn load_relation_fresh<R>(&self, source_id: &str) -> Result<(R::Source, Vec<R::Target>)>
    where
        R: RelationTrait + 'static + Send + Sync,
        R::Source: TableTrait + HasId + Send + Sync + 'static,
        R::Target: TableTrait + HasId + Send + Sync + 'static,
    {
        let def = R::def();
        let sql = format!(
            "SELECT *, ->{}->{} AS rel_targets FROM {}:{}",
            def.relation_name(),
            R::Target::table_name(),
            R::Source::table_name(),
            source_id
        );
        
        let query = Query::begin()
            .raw(
                &format!(
                    "SELECT *, ->{}->{} AS rel_targets FROM {}:{}",
                    def.relation_name(),
                    R::Target::table_name(),
                    R::Source::table_name(),
                    source_id
                )
            ).commit().build();

        let mut results: Vec<Value> = self.db.execute_raw(sql).await?;
        if results.is_empty() {
            return Err(anyhow!("No source found"));
        }

        let row = results.remove(0);
        let source: R::Source = serde_json::from_value(row.clone())?;
        let targets: Vec<R::Target> = match row.get("rel_targets") {
            Some(val) => serde_json::from_value(val.clone())?,
            None => vec![],
        };

        // Cache entities
        self.cache
            .cache_entity(R::Source::table_name(), source_id, &source)
            .await?;
        let target_ids: Vec<_> = targets.iter().map(|t| t.id().to_string()).collect();
        for t in &targets {
            self.cache
                .cache_entity(R::Target::table_name(), &t.id().to_string(), t)
                .await?;
        }

        // Cache relation IDs
        self.cache
            .cache_relation_ids(&def, source_id.parse()?, target_ids)
            .await?;

        Ok((source, targets))
    }

    pub async fn persist<T>(&self, entity: T) -> Result<()>
    where
        T: TableTrait + HasId + Send + Sync + 'static,
    {
        let mut uow = self.unit_of_work.write().await;
        uow.mark_new(entity);
        Ok(())
    }

    pub async fn manage<T>(&self, entity: T) -> Result<()>
    where
        T: TableTrait + HasId + Send + Sync + 'static,
    {
        let mut uow = self.unit_of_work.write().await;
        uow.mark_managed(entity);
        Ok(())
    }

    pub async fn remove<T>(&self, entity: T) -> Result<()>
    where
        T: TableTrait + HasId + Send + Sync + 'static,
    {
        let mut uow = self.unit_of_work.write().await;
        uow.mark_removed(entity);
        Ok(())
    }

    pub async fn flush(&self) -> Result<()> {
        let mut uow = self.unit_of_work.write().await;

        // Iterate through states and use the registered flushers
        for state in uow.states.values_mut() {
            // Safety: We verify the type through registry before flushing
            unsafe {
                state.flush_with_db(&self.db)?;
            }
        }

        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        self.unit_of_work.write().await.states.clear();
        self.cache.clear().await;
        Ok(())
    }

    pub fn cache(&self) -> Arc<EntityCache> {
        self.cache.clone()
    }
}
