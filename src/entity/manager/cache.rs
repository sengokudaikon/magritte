use crate::RelationDef;
use anyhow::{anyhow, Result};
use moka::future::Cache;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::OnceCell;

type EntityKey = String;
// (from_table, via_edge, to_table)
type RelationKey = (String, String, String);

pub struct EntityCache {
    entities: Cache<EntityKey, Arc<Value>>,
    relations: Cache<RelationKey, Vec<String>>,
}

impl Default for EntityCache {
    fn default() -> Self {
        Self::new()
    }
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
    /// The key is (from_record, via_edge, to_record) where:
    /// - from_record is the full record identifier (table:id)
    /// - via_edge is the edge type
    /// - to_record is the full record identifier (table:id)
    pub async fn cache_relation_ids(
        &self,
        def: &RelationDef,
        source_id: String,
        relation_ids: Vec<String>,
    ) -> Result<()> {
        let key = (
            format!("{}:{}", def.relation_from(), source_id),
            def.via.clone(),
            def.relation_to().to_string(),
        );
        let existing = self.relations.get(&key).await;
        if let Some(mut existing) = existing {
            existing.extend(relation_ids);
            self.relations.insert(key, existing).await;
        } else {
            self.relations.insert(key, relation_ids).await;
        }
        Ok(())
    }

    /// Retrieve cached related entity IDs for a given relation.
    pub async fn get_related_ids(&self, def: &RelationDef) -> Result<Option<Vec<String>>> {
        let key = (
            def.relation_from().to_string(), // Already contains table:id
            def.via.clone(),
            def.relation_to().to_string(), // Already contains table:id
        );
        Ok(self.relations.get(&key).await)
    }

    /// Get all relations for a given source record
    pub async fn get_relations_for_source(
        &self,
        source_record: &str, // Full record identifier (table:id)
    ) -> Result<Vec<(String, String, Vec<String>)>> {
        // (edge_type, target_record, related_ids)
        let mut result = Vec::new();

        // Scan through relations to find all that match the source record
        for (key, relations) in self.relations.iter() {
            if key.0 == source_record {
                if let Some(ids) = self.relations.get(&key).await {
                    // Return (edge_type, target_record, related_ids)
                    result.push((key.1.clone(), key.2.clone(), ids.clone()));
                }
            }
        }

        Ok(result)
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
