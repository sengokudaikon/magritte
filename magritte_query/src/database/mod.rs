pub mod executor;
use crate::database::executor::core::config::{BatchConfig, PoolConfig, QueryConfig};
use crate::database::executor::{
    BaseExecutor, ExecutorConfig, ExecutorMetrics, QueryRequest, Executor,
};
use anyhow::{anyhow, Result};
pub(crate) use deadpool_surrealdb::Config as DbConfig;
use deadpool_surrealdb::Runtime;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{channel, Sender};

/// Main database interface that handles connection management and query execution.
/// Users should not interact with this directly, but through Query builders.
#[derive(Clone)]
pub struct SurrealDB {
    executor: Arc<Executor>,
    config: DbConfig,
}

impl SurrealDB {
    /// Create a new database instance with the given configuration
    pub async fn new(config: DbConfig) -> Result<Arc<Self>> {
        // Create connection pool
        let pool = config
            .create_pool(Some(Runtime::Tokio1))
            .map_err(anyhow::Error::from)?;

        // Create executor config
        let executor_config = ExecutorConfig {
            pool: PoolConfig {
                min_connections: 3,
                max_connections: config.max_connections as usize,
                connection_timeout: Duration::from_secs(config.connect_timeout),
                idle_timeout: Default::default(),
                max_lifetime: Default::default(),
            },
            query: QueryConfig {
                query_timeout: Duration::from_secs(config.idle_timeout),
                max_retries: 5,
                retry_backoff: Default::default(),
                max_concurrent_queries: 10,
            },
            batch: BatchConfig {
                max_write_batch_size: 100,
                max_read_batch_size: 1000,
                batch_timeout: Duration::from_millis(50),
            },
        };

        // Create the structured concurrency executor
        let executor = Executor::new(pool, executor_config).await?;

        Ok(Arc::new(Self { executor, config }))
    }

    /// Internal method to execute queries from Query builders.
    /// This is not public API - users should use Query builders instead.
    pub(crate) async fn execute<T>(
        &self,
        query: String,
        params: Vec<(String, Value)>,
        query_type: QueryType,
        table_name: Option<String>,
    ) -> Result<Vec<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let (tx, mut rx) = channel(1);
        let request = QueryRequest {
            query,
            params,
            priority: QueryPriority::Normal,
            query_type,
            table_name,
            response_tx: tx,
        };

        // Execute via executor and get raw value
        self.executor
            .execute_raw(request)
            .await
            .map_err(anyhow::Error::from)?;

        // Wait for response from channel
        let result = rx.recv()
            .await
            .ok_or_else(|| anyhow!("Channel closed"))??;

        // Handle both single value and array responses
        let values = match result {
            Value::Array(arr) => arr,
            value => vec![value],
        };

        // Deserialize each value
        values
            .into_iter()
            .map(|v| {
                serde_json::from_value(v).map_err(|e| anyhow!("Deserialization failed: {}", e))
            })
            .collect()
    }

    pub async fn execute_raw<T>(&self, query: String) -> Result<Vec<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let (tx, mut rx) = channel(1);
        let request = QueryRequest {
            query,
            params: vec![],
            priority: QueryPriority::Normal,
            query_type: QueryType::Write,
            table_name: None,
            response_tx: tx,
        };

        self.executor
            .execute_raw(request)
            .await
            .map_err(anyhow::Error::from)?;

        // Wait for response from channel
        let result = rx.recv()
            .await
            .ok_or_else(|| anyhow!("Channel closed"))??;

        // Handle both single value and array responses
        let values = match result {
            Value::Array(arr) => arr,
            value => vec![value],
        };
        values
            .into_iter()
            .map(|v| {
                serde_json::from_value(v).map_err(|e| anyhow!("Deserialization failed: {}", e))
            })
            .collect()
    }

    /// Get current executor metrics (internal use)
    pub(crate) async fn metrics(&self) -> Result<Arc<ExecutorMetrics>> {
        Ok(self.executor.metrics().await)
    }
}

unsafe impl Send for SurrealDB {}

unsafe impl Sync for SurrealDB {}

/// Query priority levels for scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QueryPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Query type for scheduling decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Read,
    Write,
    Schema,
}

/// A scheduled query with metadata
#[derive(Debug)]
pub struct ScheduledQuery {
    pub query: String,
    pub params: Vec<(String, Value)>,
    pub priority: QueryPriority,
    pub query_type: QueryType,
    pub table_name: Option<String>,
    pub response_tx: Sender<Result<Value>>,
}

impl ScheduledQuery {
    pub fn new(
        query: String,
        params: Vec<(String, Value)>,
        priority: QueryPriority,
        query_type: QueryType,
        table_name: Option<String>,
        response_tx: Sender<Result<Value>>,
    ) -> Self {
        Self {
            query,
            params,
            priority,
            query_type,
            table_name,
            response_tx,
        }
    }
}
