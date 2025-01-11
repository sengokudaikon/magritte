mod executor;
mod runtime;
mod rw;
mod scheduler;
pub use scheduler::QueryType;
use crate::database::executor::{BaseExecutor, ExecutorConfig, ExecutorMetrics, QueryRequest};
use crate::database::runtime::{RuntimeConfig, RuntimeManager};
use crate::database::scheduler::QueryPriority;
use anyhow::{anyhow, bail, Result};
use async_channel::bounded;
pub(crate) use deadpool_surrealdb::Config as DbConfig;
use deadpool_surrealdb::Runtime;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use cfg_if::cfg_if;

/// Main database interface that handles connection management and query execution.
/// Users should not interact with this directly, but through Query builders.
pub struct SurrealDB {
    executor: Arc<dyn BaseExecutor + Send + Sync>,
    config: DbConfig,
}

impl SurrealDB {
    /// Create a new database instance with the given configuration
    pub async fn new(config: DbConfig) -> Result<Arc<Self>> {
        // Initialize runtime manager
        let runtime = Arc::new(RuntimeManager::new(RuntimeConfig::default()));

        // Create connection manager based on runtime
        let pool = config
            .create_pool(Some(Runtime::Tokio1))
            .map_err(anyhow::Error::from)?;

        // Create executor config
        let executor_config = ExecutorConfig {
            max_connections: config.max_connections as usize,
            connection_timeout: Duration::from_secs(config.connect_timeout),
            query_timeout: Duration::from_secs(config.idle_timeout),
            use_prepared_statements: true,
        };

        // Create and start executor
        let executor = || -> Result<Arc<dyn BaseExecutor + Send + Sync>>{
            #[cfg(feature = "rt-tokio")]
            {
                return Ok(Arc::new(
                    executor::tokio_executor::TokioExecutor::new(executor_config, pool, runtime)?,
                ));
            }

            #[cfg(feature = "rt-async-std")]
            {
                return Ok(Arc::new(
                    executor::async_std_executor::AsyncStdExecutor::new(executor_config, pool, runtime)?,
                ));
            }
        }()?;

        // Start executor
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
    ) -> Result<Vec<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let (response_tx, response_rx) = bounded(1);

        let request = QueryRequest {
            query,
            params,
            priority: QueryPriority::Normal,
            query_type, // Use the provided query type
            response_tx,
        };

        // Execute via executor
        let value = self.executor.execute_raw(request).await?;

        // Deserialize result
        serde_json::from_value(value).map_err(|e| anyhow!(e))
    }

    /// Get current executor metrics (internal use)
    pub(crate) async fn metrics(&self) -> Result<ExecutorMetrics> {
        Ok(self.executor.metrics().await)
    }
}
