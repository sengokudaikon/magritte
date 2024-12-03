//! INSERT query operations
//!
//! This module contains operations related to inserting records into tables.

use std::fmt::{Debug, Display};
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::instrument;

use crate::backend::QueryBuilder;
use crate::query_result::FromTarget;
use crate::types::{NamedType, RecordType, ReturnType, TableType};
use crate::SurrealDB;

#[derive(Debug, Clone, PartialEq)]
pub enum Content {
    /// Direct content value
    Value(serde_json::Value),
    /// SET operations
    Set(Vec<(String, serde_json::Value)>),
}
/// INSERT query builder with allowed method chains
#[derive(Clone, Debug, PartialEq)]
pub struct InsertStatement<T>
where
    T: RecordType
{
    pub(crate) targets: Option<Vec<FromTarget<T>>>,
    pub(crate) only: bool,
    pub(crate) content: Option<Content>,
    pub(crate) parameters: Vec<(String, serde_json::Value)>,
    pub(crate) parallel: bool,
    pub(crate) timeout: Option<Duration>,
    pub(crate) return_type: ReturnType,
    pub(crate) as_relation: bool,
    pub(crate) ignore: bool,
    phantom: std::marker::PhantomData<T>,
}

impl<T> InsertStatement<T>
where
    T: RecordType
{
    pub fn content(mut self, content: T) -> Result<Self> {
        self.content = Some(Content::Value(serde_json::to_value(content)?));
        Ok(self)
    }
    #[instrument(skip_all)]
    pub fn values<C>(mut self, values: Vec<C>) -> Result<Self>
    where
        C: Serialize,
    {
        let values = values
            .into_iter()
            .map(|v| serde_json::to_value(v))
            .collect::<Result<Vec<_>, _>>()?;
        self.content = Some(Content::Value(serde_json::Value::Array(values)));
        Ok(self)
    }

    /// Add ON DUPLICATE KEY UPDATE clause
    #[instrument(skip_all)]
    pub fn on_duplicate_key_update<V: Serialize>(mut self, field: &str, value: V) -> Result<Self> {
        match &mut self.content {
            Some(Content::Set(sets)) => {
                sets.push((field.to_string(), serde_json::to_value(value)?));
            }
            _ => {
                self.content = Some(Content::Set(vec![(
                    field.to_string(),
                    serde_json::to_value(value)?,
                )]));
            }
        }
        Ok(self)
    }

    /// Add ON DUPLICATE KEY UPDATE with increment
    #[instrument(skip_all)]
    pub fn on_duplicate_key_increment(mut self, field: &str, value: i64) -> Result<Self> {
        let expr = format!("{} += {}", field, value);
        match &mut self.content {
            Some(Content::Set(sets)) => {
                sets.push((expr, serde_json::Value::Null));
            }
            _ => {
                self.content = Some(Content::Set(vec![(expr, serde_json::Value::Null)]));
            }
        }
        Ok(self)
    }

    /// Add ON DUPLICATE KEY UPDATE with decrement
    #[instrument(skip_all)]
    pub fn on_duplicate_key_decrement(mut self, field: &str, value: i64) -> Result<Self> {
        let expr = format!("{} -= {}", field, value);
        match &mut self.content {
            Some(Content::Set(sets)) => {
                sets.push((expr, serde_json::Value::Null));
            }
            _ => {
                self.content = Some(Content::Set(vec![(expr, serde_json::Value::Null)]));
            }
        }
        Ok(self)
    }

    /// Insert as a relation
    #[instrument(skip(self))]
    pub fn as_relation(mut self) -> Self {
        self.as_relation = true;
        self
    }

    /// Insert with IGNORE option
    #[instrument(skip(self))]
    pub fn ignore(mut self) -> Self {
        self.ignore = true;
        self
    }
}
#[async_trait]
impl<T> QueryBuilder<T> for InsertStatement<T>
where
    T: RecordType
{
    fn new() -> Self {
        Self {
            targets: None,
            only: false,
            content: None,
            parameters: vec![],
            parallel: false,
            timeout: None,
            return_type: Default::default(),
            as_relation: false,
            ignore: false,
            phantom: std::marker::PhantomData,
        }
    }

    fn build(&self) -> Result<String> {
        todo!()
    }

    async fn execute(self, _conn: SurrealDB) -> Result<Vec<T>> {
        todo!()
    }
}
