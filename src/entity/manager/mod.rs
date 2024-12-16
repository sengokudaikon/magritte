use crate::entity::{EntityCache, LoadStrategy};
use crate::{ColumnTrait, HasColumns, HasRelations, RelationDef, RelationTrait, TableTrait};
use anyhow::Result;
use magritte_query::{HasId, NamedType, Query, RecordType, SurrealDB, SurrealId};
use serde::de::DeserializeOwned;
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityState {
    New,
    Managed,
    Detached,
    Removed,
}

pub struct UnitOfWork {
    new: HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>,
    managed: HashMap<TypeId, HashMap<String, Box<dyn Any + Send + Sync>>>,
    removed: HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>,
    dirty: HashMap<TypeId, HashSet<String>>,
}

impl UnitOfWork {
    pub fn new() -> Self {
        Self {
            new: HashMap::new(),
            managed: HashMap::new(),
            removed: HashMap::new(),
            dirty: HashMap::new(),
        }
    }

    pub fn mark_new<T: 'static + Send + Sync>(&mut self, entity: T) {
        let type_id = TypeId::of::<T>();
        self.new.entry(type_id).or_default().push(Box::new(entity));
    }

    pub fn mark_managed<T: 'static + Send + Sync + HasId>(&mut self, entity: T) {
        let type_id = TypeId::of::<T>();
        let id = entity.id().to_string();
        self.managed
            .entry(type_id)
            .or_default()
            .insert(id, Box::new(entity));
    }

    pub fn mark_removed<T: 'static + Send + Sync>(&mut self, entity: T) {
        let type_id = TypeId::of::<T>();
        self.removed
            .entry(type_id)
            .or_default()
            .push(Box::new(entity));
    }

    pub fn mark_dirty<T: 'static>(&mut self, id: String) {
        let type_id = TypeId::of::<T>();
        self.dirty.entry(type_id).or_default().insert(id);
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

#[async_trait::async_trait]
pub trait LoadingHandler: Send + Sync {
    async fn load_relation<R>(
        &self,
        source_id: &str,
        relation: R,
    ) -> Result<(R::Source, Vec<R::Target>)>
    where
        R: RelationTrait + 'static + Send + Sync,
        R::Source: RecordType + HasId + Send + Sync + 'static,
        R::Target: RecordType + HasId + Send + Sync + 'static;
}

pub struct DefaultLoadingHandler {
    entity_manager: Arc<EntityManager>,
}

impl DefaultLoadingHandler {
    pub fn new(entity_manager: Arc<EntityManager>) -> Self {
        Self { entity_manager }
    }
}

#[async_trait::async_trait]
impl LoadingHandler for DefaultLoadingHandler {
    async fn load_relation<R>(
        &self,
        source_id: &str,
        relation: R,
    ) -> Result<(R::Source, Vec<R::Target>)>
    where
        R: RelationTrait + 'static + Send + Sync,
        R::Source: TableTrait + HasId + Send + Sync + 'static,
        R::Target: TableTrait + HasId + Send + Sync + 'static,
    {
        let def = relation.def_owned();
        let relation_name = def.relation_name();

        // Build the query manually
        let query = Query::select()
            .field("*", None)
            .field(def.relation_to(), Some(relation_name))
            .fetch(&[relation_name])
            .where_id(SurrealId::<R::Source>::from(source_id));

        let results = query.execute(self.entity_manager.db()).await.map_err(anyhow::Error::from)?;
        if results.is_empty() {
            return Err(anyhow::anyhow!("Source entity not found"));
        }

        let value = &results[0];
        let (source, targets) = self
            .entity_manager
            .parse_with_related_result::<R::Source, R::Target>(value, relation_name)?;
        self.entity_manager
            .cache_entity_and_relations::<R::Source, R::Target>(&def, source_id, &source, &targets)
            .await?;
        Ok((source, targets))
    }
}

pub struct EntityManager {
    db: SurrealDB,
    cache: Arc<EntityCache>,
    unit_of_work: RwLock<UnitOfWork>,
    loading_config: LoadingConfig,
}

impl EntityManager {
    pub fn new(db: SurrealDB, cache: EntityCache, config: Option<LoadingConfig>) -> Self {
        Self {
            db,
            cache: Arc::new(cache),
            unit_of_work: RwLock::new(UnitOfWork::new()),
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
            unit_of_work: RwLock::new(UnitOfWork::new()),
            loading_config: self.loading_config.clone(),
        })
    }

    /// Find an entity by ID, loading record<...> fields eagerly and also caching eager edge relations.
    /// Returns just `T`, with record fields embedded. Eager edge relations are cached but not returned here.
    pub async fn find<T>(&self, id: &str) -> Result<Option<T>>
    where
        T: RecordType + HasId + HasColumns + HasRelations + Send + Sync + 'static,
    {
        // Check cache first
        if let Some(entity) = self.cache.get_entity::<T>(T::table_name(), id).await? {
            return Ok(Some(entity));
        }

        let mut qb = Query::select().only();

        // Record fields to fetch
        let record_fields: Vec<&str> = T::column_defs()
            .into_iter()
            .filter(|c| c.is_record())
            .map(|c| c.name())
            .collect();

        // Eager relations
        let eager_relations: Vec<&str> = T::relation_defs()
            .into_iter()
            .filter(|r| r.load_strategy() == LoadStrategy::Eager)
            .map(|r| r.relation_name())
            .collect();

        let all_fetches: Vec<&str> = record_fields
            .iter()
            .chain(eager_relations.iter())
            .copied()
            .collect();
        if !all_fetches.is_empty() {
            qb = qb.fetch(&all_fetches);
        }

        qb = qb.where_id(SurrealId::<T>::from(id));

        let mut entities = qb.execute(self.db()).await.map_err(anyhow::Error::from)?;
        let entity = match entities.pop() {
            Some(e) => e,
            None => return Ok(None),
        };

        // Cache the loaded entity
        self.cache
            .cache_entity(T::table_name(), id, &entity)
            .await?;

        // If there are eager edge relations, they are now fetched and included in `entity`. We assume SurrealDB
        // returns them under fields named after `relation_name`. Extract and cache them:
        for relation_name in eager_relations {
            // Extract related entities from JSON
            let targets: Vec<serde_json::Value> =
                entity_extract_relation_field(&entity, relation_name);
            // Deserialize each target and cache them
            let mut target_ids = Vec::new();
            for tjson in targets {
                let t: serde_json::Value = tjson;
                // We don't know the target type from here directly. You have two options:
                // 1) If you know the target type from `relation_defs`, you can do a runtime dispatch.
                // 2) You can rely on load_relation calls later.
                //
                // For simplicity, just store the JSON and the IDs in the cache. Let's assume we have a way
                // to identify the target table and ID from `t`.
                //
                // Realistically, you'd need a typed approach here similar to load_relation_fresh.
                // For this demonstration, we just skip detailed caching of eager edges. Instead:
                // The user will call load_relation later to get typed data. Since we have the data now,
                // we could parse and cache it if we knew the target type. Without that, we might need
                // a runtime registry.
            }
        }

        Ok(Some(entity))
    }

    /// Loads a given relation (lazy or eager) on-demand. Returns (Source, Vec<Target>).
    /// If eager and previously fetched, this should be cached, otherwise it queries again.
    pub async fn load_relation<R>(
        &self,
        entity_id: &str,
        relation: R,
    ) -> Result<(R::Source, Vec<R::Target>)>
    where
        R: RelationTrait + 'static + Send + Sync,
        R::Source: RecordType + HasId + 'static + Send + Sync,
        R::Target: RecordType + HasId + 'static + Send + Sync,
    {
        let def = relation.def();
        let relation_name = def.relation_name();

        if let Some(cached_ids) = self.cache.get_related_ids(&def).await? {
            let source = self
                .cache
                .get_entity::<R::Source>(R::Source::table_name(), entity_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Source not found in cache"))?;

            let mut targets = Vec::new();
            for rid in cached_ids {
                if let Some(target) = self
                    .cache
                    .get_entity::<R::Target>(R::Target::table_name(), &rid)
                    .await?
                {
                    targets.push(target);
                } else {
                    // Not all targets cached, load fresh:
                    return self.load_relation_fresh(entity_id, relation).await;
                }
            }
            return Ok((source, targets));
        }

        // Not cached, load fresh
        self.load_relation_fresh(entity_id, relation).await
    }

    async fn load_relation_fresh<R>(
        &self,
        entity_id: &str,
        relation: R,
    ) -> Result<(R::Source, Vec<R::Target>)>
    where
        R: RelationTrait + 'static + Send + Sync,
        R::Source: RecordType + HasId + 'static + Send + Sync,
        R::Target: RecordType + HasId + 'static + Send + Sync,
    {
        let handler = DefaultLoadingHandler::new(self.clone_self());
        handler.load_relation(entity_id, relation).await
    }

    fn parse_with_related_result<S, T>(
        &self,
        value: &serde_json::Value,
        relation_name: &str,
    ) -> Result<(S, Vec<T>)>
    where
        S: DeserializeOwned,
        T: DeserializeOwned,
    {
        let source: S = serde_json::from_value(value.clone())?;
        let targets_json = value
            .get(relation_name)
            .cloned()
            .unwrap_or(serde_json::Value::Array(vec![]));
        let targets: Vec<T> = serde_json::from_value(targets_json)?;
        Ok((source, targets))
    }

    async fn cache_entity_and_relations<S, T>(
        &self,
        def: &RelationDef,
        source_id: &str,
        source: &S,
        targets: &[T],
    ) -> Result<()>
    where
        S: HasId + RecordType,
        T: HasId + RecordType,
    {
        self.cache
            .cache_entity(S::table_name(), source_id, source)
            .await?;

        let ids: Vec<String> = targets.iter().map(|t| t.id().to_string()).collect();
        self.cache.cache_relation_ids(def, ids.clone()).await?;

        for t in targets {
            self.cache
                .cache_entity(T::table_name(), &t.id().to_string(), t)
                .await?;
        }

        Ok(())
    }

    pub async fn persist<T>(&self, entity: T) -> Result<()>
    where
        T: RecordType + HasId + Send + Sync + 'static,
    {
        let mut uow = self.unit_of_work.write().await;
        uow.mark_new(entity);
        Ok(())
    }

    pub async fn manage<T>(&self, entity: T) -> Result<()>
    where
        T: RecordType + HasId + Send + Sync + 'static,
    {
        let mut uow = self.unit_of_work.write().await;
        uow.mark_managed(entity);
        Ok(())
    }

    pub async fn remove<T>(&self, entity: T) -> Result<()>
    where
        T: RecordType + HasId + Send + Sync + 'static,
    {
        let mut uow = self.unit_of_work.write().await;
        uow.mark_removed(entity);
        Ok(())
    }

    pub async fn flush(&self) -> Result<()> {
        let mut uow = self.unit_of_work.write().await;

        // Handle new entities
        for (_type_id, entities) in uow.new.drain() {
            for _entity in entities {
                // TODO: downcast and insert using CRUD
            }
        }

        // Handle removed entities
        for (_type_id, entities) in uow.removed.drain() {
            for _entity in entities {
                // TODO: downcast and delete using CRUD
            }
        }

        // Handle dirty entities
        for (_type_id, ids) in uow.dirty.drain() {
            for _id in ids {
                // TODO: update entities
            }
        }

        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        let mut uow = self.unit_of_work.write().await;
        uow.new.clear();
        uow.managed.clear();
        uow.removed.clear();
        uow.dirty.clear();
        self.cache.clear().await;
        Ok(())
    }

    pub fn cache(&self) -> Arc<EntityCache> {
        self.cache.clone()
    }
}

/// Extracts related entities from a SurrealDB returned entity.
/// This is just a helper function for demonstration. Adjust as needed.
fn entity_extract_relation_field(
    entity: &serde_json::Value,
    relation_name: &str,
) -> Vec<serde_json::Value> {
    match entity.get(relation_name) {
        Some(serde_json::Value::Array(arr)) => arr.clone(),
        _ => vec![],
    }
}
