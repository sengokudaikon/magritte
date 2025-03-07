pub mod executor;
use crate::database::executor::utils::metrics::ExecutorMetrics;
use anyhow::Result;
pub(crate) use deadpool_surrealdb::Config as DbConfig;
use deadpool_surrealdb::Runtime;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;
pub(crate) use crate::database::executor::core::types::QueryType;

/// Main database interface that handles connection management and query execution.
/// Users should not interact with this directly, but through Query builders.
#[derive(Clone)]
pub struct SurrealDB {
    pool: deadpool_surrealdb::Pool,
    metrics: Arc<ExecutorMetrics>,
}

impl SurrealDB {
    /// Create a new database instance with the given configuration
    pub async fn new(config: DbConfig) -> Result<Arc<Self>> {
        // Create connection pool
        let pool = config
            .create_pool(Some(Runtime::Tokio1))
            .map_err(anyhow::Error::from)?;
        let metrics = Arc::new(ExecutorMetrics::new());
        Ok(Arc::new(Self { pool, metrics }))
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
        let conn = self.pool.get().await?;
        let metrics = self.metrics.clone();
        let start = std::time::Instant::now();
        let result = {
            let mut q = conn.query(&query);
            if !params.is_empty() {
                q = q.bind(params)
            }
            q.await
        };

        match result {
            Ok(mut response) => {
                metrics.update_success(start.elapsed().as_micros() as usize);
                response.take(0).map_err(anyhow::Error::from)
            }
            Err(e) => {
                metrics.update_failure();
                Err(anyhow::anyhow!(e))
            }
        }
    }
}

unsafe impl Send for SurrealDB {}

unsafe impl Sync for SurrealDB {}
