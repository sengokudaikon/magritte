mod executor;
mod runtime;
mod scheduler;
pub use scheduler::QueryType;
use crate::database::executor::{BaseExecutor, ExecutorConfig, ExecutorMetrics, QueryRequest};
use crate::database::runtime::{RuntimeConfig, RuntimeManager};
use crate::database::scheduler::QueryPriority;
use anyhow::{anyhow, Result};
use async_channel::bounded;
pub(crate) use deadpool_surrealdb::Config as DbConfig;
use deadpool_surrealdb::Runtime;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use crate::database::executor::future_executor::FutureExecutor;

/// Main database interface that handles connection management and query execution.
/// Users should not interact with this directly, but through Query builders.
#[derive(Clone)]
pub struct SurrealDB {
    executor: Arc<dyn BaseExecutor>,
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
            max_connections: config.max_connections as usize,
            connection_timeout: Duration::from_secs(config.connect_timeout),
            query_timeout: Duration::from_secs(config.idle_timeout),
            use_prepared_statements: true,
            max_batch_size: 100, // Default batch size
            batch_timeout_ms: 50, // Default batch timeout
        };

        // Create and start executor
        let executor = FutureExecutor::new(
            executor_config, 
            pool,
            Arc::new(RuntimeManager::new(RuntimeConfig::default())),
        )?;
        
        // Wrap in Arc and start
        let executor: Arc<dyn BaseExecutor> = Arc::new(executor);
        executor.run().await?;

        Ok(Arc::new(Self { executor, config }))
    }

    /// Internal method to execute queries from Query builders.
    /// This is not public API - users should use Query builders instead.
    pub(crate) async fn execute<T>(
        &self,
        query: String,
        params: Vec<(String, Value)>,
        query_type: QueryType,
        table_name: Option<String>
    ) -> Result<Vec<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let request = QueryRequest {
            query,
            params,
            priority: QueryPriority::Normal,
            query_type,
            table_name,
            response_tx: bounded(1).0,
        };

        // Execute via executor and get raw value
        let raw_value = self.executor.execute_raw(request).await?;
        
        // Handle both single value and array responses
        let values = match raw_value {
            Value::Array(arr) => arr,
            value => vec![value],
        };

        // Deserialize each value
        values
            .into_iter()
            .map(|v| serde_json::from_value(v).map_err(|e| anyhow!("Deserialization failed: {}", e)))
            .collect()
    }

    /// Get current executor metrics (internal use)
    pub(crate) async fn metrics(&self) -> Result<Arc<ExecutorMetrics>> {
        Ok(self.executor.metrics().await)
    }
}
