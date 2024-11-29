//! UPDATE query operations
//!
//! This module contains operations related to updating existing records in
//! tables.

use std::marker::PhantomData;
use anyhow::Result;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::instrument;

use crate::backend::QueryBuilder;
use crate::types::TableType;
use crate::SurrealDB;
#[derive(Clone, Debug, PartialEq)]
pub enum Content {
    /// MERGE operations (for UPDATE)
    Merge(serde_json::Value),
    /// PATCH operations (for UPDATE)
    Patch(serde_json::Value),
}
#[derive(Clone, Debug, PartialEq)]
pub struct UpdateStatement<T> {
    content: Option<Content>,
    _marker: PhantomData<T>,
}

impl<T> UpdateStatement<T>
where
    T: TableType + Serialize + DeserializeOwned,
{
    /// Add a MERGE operation for UPDATE
    #[instrument(skip_all)]
    pub fn merge<C: Serialize>(mut self, content: C) -> Result<Self> {
        self.content = Some(Content::Merge(serde_json::to_value(content)?));
        Ok(self)
    }

    /// Add a PATCH operation for UPDATE
    #[instrument(skip_all)]
    pub fn patch<C: Serialize>(mut self, patch: C) -> Result<Self> {
        self.content = Some(Content::Patch(serde_json::to_value(patch)?));
        Ok(self)
    }
}

#[async_trait]
impl<T> QueryBuilder<T> for UpdateStatement<T>
where
    T: TableType + Serialize + DeserializeOwned,
{
    fn new() -> Self {
        Self { content: None,
            _marker: PhantomData,
        }
    }

    fn build(&self) -> Result<String> {
        todo!()
    }

    async fn execute(self, _conn: SurrealDB) -> Result<Vec<T>> {
        todo!()
    }
}
