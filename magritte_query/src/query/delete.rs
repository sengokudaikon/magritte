//! DELETE query operations
//!
//! This module contains operations related to deleting records from tables.

use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::time::Duration;

use crate::{
    FromTarget, HasConditions, HasParams, Operator,  RangeTarget, RecordType,
    ReturnType, Returns, SqlValue, SurrealDB, SurrealId, WhereClause,
};
use anyhow::{anyhow, bail};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use tracing::{error, info, instrument};

#[derive(Clone, Debug, PartialEq)]
pub struct DeleteStatement<T>
where
    T: RecordType,
{
    pub(crate) with_id: Option<SurrealId<T>>,
    pub(crate) with_range: Option<RangeTarget>,
    pub(crate) targets: Option<Vec<FromTarget<T>>>,
    pub(crate) only: bool,
    pub(crate) conditions: Vec<(String, Operator, SqlValue)>,
    pub(crate) parameters: Vec<(String, Value)>,
    pub(crate) timeout: Option<Duration>,
    pub(crate) parallel: bool,
    pub(crate) return_type: Option<ReturnType>,
    phantom: PhantomData<T>,
}
impl<T> DeleteStatement<T>
where
    T: RecordType,
{
    pub fn where_id(mut self, id: SurrealId<T>) -> Self {
        self.with_id = Some(id);
        self
    }

    pub fn range(mut self, range_target: RangeTarget) -> Self {
        self.with_range = Some(range_target);
        self
    }

    pub fn targets(mut self, targets: Vec<SurrealId<T>>) -> anyhow::Result<Self> {
        if self.only {
            Ok(self)
        } else {
            let targets = vec![FromTarget::RecordList(targets)];
            self.targets = Some(targets);
            Ok(self)
        }
    }

    /// Delete ONLY that element and return it as a response
    /// MUST USE RETURN BEFORE
    pub fn only(mut self) -> Self {
        self.only = true;
        self
    }

    /// Add a TIMEOUT duration
    #[instrument(skip_all)]
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Enable parallel execution
    #[instrument(skip(self))]
    pub fn parallel(mut self) -> Self {
        self.parallel = true;
        self
    }

    /// Start building an edge deletion
    pub fn edge_of(self, edge: &str) -> EdgeDeleteStatement<T> {
        EdgeDeleteStatement::new(self, edge)
    }

    #[instrument(skip_all)]
    pub(crate) fn new() -> Self {
        Self {
            with_id: None,
            with_range: None,
            targets: None,
            only: false,
            conditions: vec![],
            parameters: vec![],
            timeout: None,
            parallel: false,
            return_type: Default::default(),
            phantom: Default::default(),
        }
    }
    #[instrument(skip(self))]
    pub(crate) fn build(&self) -> anyhow::Result<String> {
        let mut query = String::new();
        query.push_str("DELETE ");
        if self.only {
            if let Some(return_type) = &self.return_type {
                if *return_type != ReturnType::Before {
                    bail!("When using .only(), the return type must be ReturnType::Before");
                }
            } else {
                bail!(
                    "When using .only(), a return type must be specified using .return_() methods."
                );
            }
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

        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            let conditions: Vec<String> = self
                .conditions
                .iter()
                .map(|(field, op, value)| {
                    format!("{} {} {}", field, String::from(op.clone()), value)
                })
                .collect();
            query.push_str(&conditions.join(" AND "));
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
        if let Some(timeout) = &self.timeout {
            query.push_str(&format!(" TIMEOUT {}", timeout.as_secs()));
        }
        if self.parallel {
            query.push_str(" PARALLEL");
        }
        query.push(';');
        Ok(query)
    }
    #[instrument(skip_all)]
    async fn execute(self, conn: SurrealDB) -> anyhow::Result<Vec<T>> {
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
impl<T> HasParams for DeleteStatement<T>
where
    T: RecordType,
{
    fn params(&self) -> &Vec<(String, Value)> {
        &self.parameters
    }

    fn params_mut(&mut self) -> &mut Vec<(String, Value)> {
        &mut self.parameters
    }
}
impl<T> HasConditions for DeleteStatement<T>
where
    T: RecordType,
{
    fn conditions_mut(&mut self) -> &mut Vec<(String, Operator, SqlValue)> {
        &mut self.conditions
    }
}
impl<T> Returns for DeleteStatement<T>
where
    T: RecordType,
{
    fn return_type_mut(&mut self) -> &mut Option<ReturnType> {
        &mut self.return_type
    }
}

/// Builder for edge deletion operations
pub struct EdgeDeleteStatement<T>
where
    T: RecordType,
{
    inner: DeleteStatement<T>,
    edge: String,
    from_id: Option<String>,
    to_id: Option<String>,
}

impl<T> EdgeDeleteStatement<T>
where
    T: RecordType,
{
    #[instrument(skip(inner))]
    pub fn new(inner: DeleteStatement<T>, edge: &str) -> Self {
        Self {
            inner,
            edge: edge.to_string(),
            from_id: None,
            to_id: None,
        }
    }
    #[instrument(skip(self))]
    pub fn from(mut self, id: &str) -> Self {
        self.from_id = Some(id.to_string());
        self
    }
    #[instrument(skip(self))]
    pub fn to(mut self, id: &str) -> Self {
        self.to_id = Some(id.to_string());
        self
    }
    #[instrument(skip(self))]
    pub fn where_op<V: Serialize + Debug>(
        mut self,
        field: &str,
        op: Operator,
        value: Option<V>,
    ) -> anyhow::Result<Self> {
        self.inner = self.inner.where_op(field, op, value)?;
        Ok(self)
    }
    #[instrument(skip(self))]
    pub fn return_(mut self, return_type: ReturnType) -> Self {
        self.inner = self.inner.return_(return_type);
        self
    }
    #[instrument(skip(self))]
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.inner = self.inner.timeout(duration);
        self
    }
    #[instrument(skip(self))]
    pub fn parallel(mut self) -> Self {
        self.inner = self.inner.parallel();
        self
    }
    #[instrument(skip(self))]
    pub fn build(self) -> anyhow::Result<String> {
        let mut query = String::new();
        query.push_str("DELETE ");

        // Add FROM part
        query.push_str(T::table_name());
        if let Some(from) = self.from_id {
            query.push(':');
            query.push_str(&from);
        }

        // Add EDGE part
        query.push_str("->");
        query.push_str(&self.edge);

        // Add TO part if specified
        if let Some(to) = self.to_id {
            query.push_str("->");
            query.push_str(T::table_name());
            query.push(':');
            query.push_str(&to);
        }

        // Add WHERE clause if any
        if !self.inner.conditions.is_empty() {
            query.push_str(" WHERE ");
            let conditions: Vec<String> = self
                .inner
                .conditions
                .iter()
                .map(|(field, op, value)| {
                    format!("{} {} {}", field, String::from(op.clone()), value)
                })
                .collect();
            query.push_str(&conditions.join(" AND "));
        }

        // Add RETURN clause if specified
        if let Some(return_type) = &self.inner.return_type {
            query.push_str(" RETURN ");
            query.push_str(&return_type.to_string());
        }

        // Add TIMEOUT if specified
        if let Some(timeout) = &self.inner.timeout {
            query.push_str(&format!(" TIMEOUT {}", timeout.as_secs()));
        }

        // Add PARALLEL if enabled
        if self.inner.parallel {
            query.push_str(" PARALLEL");
        }

        query.push(';');
        Ok(query)
    }
}
