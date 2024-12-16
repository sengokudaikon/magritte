use moka::future::Cache;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::OnceCell;
use magritte_query::RecordType;
use anyhow::Result;
use crate::RelationDef;

type EntityKey = String;
type RelationKey = (String, String, String); // (from, to, via)

pub struct EntityCache {
    entities: Cache<EntityKey, Arc<Value>>,
    relations: Cache<RelationKey, Vec<String>>,
}

impl EntityCache {
    pub fn new() -> Self {
        Self {
            entities: Cache::builder()
                .time_to_live(Duration::from_secs(3600))
                .time_to_idle(Duration::from_secs(1800))
                .max_capacity(10_000)
                .build(),
            relations: Cache::builder()
                .time_to_live(Duration::from_secs(3600))
                .time_to_idle(Duration::from_secs(1800))
                .max_capacity(50_000)
                .build(),
        }
    }

    pub async fn cache_entity<T: serde::Serialize>(
        &self,
        table_name: &str,
        id: &str,
        entity: &T,
    ) -> Result<()> {
        let key = format!("{}:{}", table_name, id);
        let value = serde_json::to_value(entity)?;
        self.entities.insert(key, Arc::new(value)).await;
        Ok(())
    }

    pub async fn get_entity<T: serde::de::DeserializeOwned>(
        &self,
        table_name: &str,
        id: &str,
    ) -> Result<Option<T>> {
        let key = format!("{}:{}", table_name, id);
        if let Some(value) = self.entities.get(&key).await {
            let entity = serde_json::from_value::<T>((*value).clone())?;
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }

    /// Cache the related entity IDs for a given relation.
    /// Uses the actual relation metadata to form a stable key.
    pub async fn cache_relation_ids(
        &self,
        def: &RelationDef,
        related_ids: Vec<String>,
    ) -> Result<()> {
        let key = (
            def.relation_from().to_string(),
            def.relation_to().to_string(),
            def.relation_name().to_string(),
        );
        self.relations.insert(key, related_ids).await;
        Ok(())
    }

    /// Retrieve cached related entity IDs for a given relation.
    pub async fn get_related_ids(
        &self,
        def: &RelationDef,
    ) -> Result<Option<Vec<String>>> {
        let key = (
            def.relation_from().to_string(),
            def.relation_to().to_string(),
            def.relation_name().to_string(),
        );
        Ok(self.relations.get(&key).await.map(|ids| ids.clone()))
    }

    pub async fn clear(&self) {
        self.entities.invalidate_all();
        self.relations.invalidate_all();
    }

    pub async fn invalidate(&self, table_name: &str, id: &str) {
        let key = format!("{}:{}", table_name, id);
        self.entities.invalidate(&key).await;
    }
}

static CACHE: OnceCell<EntityCache> = OnceCell::const_new();

pub async fn global_cache() -> &'static EntityCache {
    CACHE.get_or_init(|| async { EntityCache::new() }).await
}