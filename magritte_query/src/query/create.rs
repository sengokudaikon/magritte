//! CREATE query operations
//!
//! This module contains operations related to creating new records in tables.

use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use crate::transaction::Transactional;
use crate::{FromTarget, RangeTarget, RecordType, ReturnType, Returns, SurrealId};
use anyhow::{anyhow, Result};
use serde::Serialize;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use tracing::{error, info, instrument};

#[derive(Debug, Clone)]
pub enum Content {
    /// Direct content value
    Value(serde_json::Value),
    /// SET operations
    Set(Vec<(String, serde_json::Value)>),
}

/// CREATE query builder with allowed method chains
#[derive(Clone, Debug)]
pub struct CreateStatement<T>
where
    T: RecordType,
{
    with_id: Option<String>,
    with_range: Option<RangeTarget>,
    targets: Option<Vec<FromTarget<T>>>,
    only: bool,
    content: Option<Content>,
    parameters: Vec<(String, serde_json::Value)>,
    parallel: bool,
    timeout: Option<Duration>,
    return_type: Option<ReturnType>,
    version: Option<String>,
    in_transaction: bool,
    _marker: PhantomData<T>,
}
impl<T> CreateStatement<T>
where
    T: RecordType,
{
    pub fn with_id(mut self, id: &str) -> Self {
        self.with_id = Some(id.to_string());
        self
    }

    pub fn range(mut self, range_target: RangeTarget) -> Self {
        self.with_range = Some(range_target);
        self
    }

    pub fn targets(mut self, targets: Vec<SurrealId<T>>) -> Result<Self> {
        if self.only {
            Ok(self)
        } else {
            let targets: Vec<FromTarget<T>> = vec![FromTarget::RecordList(targets)];
            self.targets = Some(targets);
            Ok(self)
        }
    }

    /// Create a new record with content
    #[instrument(skip_all)]
    pub fn content<C>(mut self, content: C) -> Result<Self>
    where
        C: Serialize,
    {
        self.content = Some(Content::Value(serde_json::to_value(content)?));
        Ok(self)
    }

    /// Add a SET operation for CREATE
    #[instrument(skip_all)]
    pub fn set<V: Serialize>(mut self, field: &str, value: V) -> Result<Self> {
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

    /// Add a TIMEOUT duration
    #[instrument(skip_all)]
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Add ONLY
    pub fn only(mut self) -> Self {
        self.only = true;
        self
    }

    /// Enable parallel execution
    #[instrument(skip(self))]
    pub fn parallel(mut self) -> Self {
        self.parallel = true;
        self
    }

    /// Add VERSION clause
    pub fn version(mut self, timestamp: &str) -> Self {
        self.version = Some(format!("d'{}'", timestamp));
        self
    }

    #[instrument(skip_all)]
    pub fn new() -> Self {
        Self {
            with_id: None,
            with_range: None,
            targets: None,
            only: false,
            content: None,
            parameters: vec![],
            parallel: false,
            timeout: None,
            return_type: Default::default(),
            version: None,
            in_transaction: false,
            _marker: PhantomData,
        }
    }
    #[instrument(skip_all)]
    fn build(&self) -> anyhow::Result<String> {
        let mut query = String::new();
        query.push_str("CREATE ");
        if self.only {
            query.push_str("ONLY ");
        } else if let Some(targets) = &self.targets {
            if !targets.is_empty() {
                for v in targets {
                    query.push_str(&format!("{},", v.to_string().as_str()));
                }
            }
        }

        if let Some(id) = &self.with_id {
            query.push_str(T::table_name());
            query.push_str(&format!(":{}", id));
        } else if let Some(range) = &self.with_range {
            query.push('|');
            query.push_str(T::table_name());
            query.push_str(&format!(":{}|", range));
        } else {
            query.push_str(T::table_name());
        }

        if let Some(content) = &self.content {
            match content {
                Content::Value(value) => {
                    query.push_str(" CONTENT ");
                    query.push_str(&value.to_string());
                }
                Content::Set(sets) => {
                    query.push_str(" SET ");
                    let set_strs: Vec<String> = sets
                        .iter()
                        .map(|(field, value)| format!("{} = {}", field, value))
                        .collect();
                    query.push_str(&set_strs.join(", "));
                }
            }
        }
        if let Some(timeout) = &self.timeout {
            query.push_str(&format!(" TIMEOUT {}", timeout.as_secs()));
        }
        if let Some(return_type) = &self.return_type {
            match return_type {
                ReturnType::All => query.push_str(" RETURN AFTER"),
                ReturnType::None => query.push_str(" RETURN NONE"),
                ReturnType::Before => query.push_str(" RETURN BEFORE"),
                ReturnType::After => query.push_str(" RETURN AFTER"),
                ReturnType::Diff => query.push_str(" RETURN DIFF"),
                ReturnType::Fields(fields) => {
                    query.push_str(" RETURN ");
                    query.push_str(&fields.join(", "));
                }
            }
        }
        if let Some(ver) = &self.version {
            query.push_str(&format!(" VERSION {}", ver));
        }
        if self.parallel {
            query.push_str(" PARALLEL");
        }
        query.push(';');
        Ok(query)
    }

    /// Execute the CREATE query
    #[instrument(skip_all)]
    async fn execute(self, conn: Arc<Surreal<Any>>) -> Result<Vec<T>> {
        let query = self.build()?;
        info!("Executing query: {}", query);
        let mut surreal_query = conn.query(query);

        // Bind all parameters
        for (name, value) in self.parameters {
            surreal_query = surreal_query.bind((name, value));
        }

        let res = surreal_query.await?.take(0);
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("Query execution failed: {:?}", e);
                Err(anyhow!(e))
            }
        }
    }
}

impl<T> Returns for CreateStatement<T>
where
    T: RecordType,
{
    fn return_type_mut(&mut self) -> &mut Option<ReturnType> {
        &mut self.return_type
    }
}
impl<T> Transactional for CreateStatement<T>
where
    T: RecordType,
{
    fn is_transaction(&self) -> bool {
        self.in_transaction
    }

    fn in_transaction(&mut self) -> &mut bool {
        &mut self.in_transaction
    }
}
