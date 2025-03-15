pub mod executor;
pub use crate::executor::core::types::QueryType;
use crate::executor::utils::metrics::ExecutorMetrics;
use anyhow::Result;
pub(crate) use deadpool_surrealdb::Config as DbConfig;
use deadpool_surrealdb::Runtime;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::{Arc, OnceLock};

/// Main database interface that handles connection management and query execution.
/// Users should not interact with this directly, but through Query builders.
#[derive(Clone)]
pub struct SurrealDB {
    pool: deadpool_surrealdb::Pool,
    metrics: Arc<ExecutorMetrics>,
}

impl SurrealDB {
    /// Create a new database instance with the given configuration
    pub fn new(config: DbConfig) -> Result<Self> {
        // Create connection pool
        let pool = config
            .create_pool(Some(Runtime::Tokio1))
            .map_err(anyhow::Error::from)?;
        let metrics = Arc::new(ExecutorMetrics::new());
        Ok(Self { pool, metrics })
    }

    /// Internal method to execute queries from Query builders.
    /// This is not public API - users should use Query builders instead.
    pub async fn execute<T>(
        &self,
        query: impl ToString,
        params: Vec<(String, Value)>,
    ) -> Result<Vec<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let conn = self.pool.get().await?;
        let metrics = self.metrics.clone();
        let start = std::time::Instant::now();
        let result = {
            let mut q = conn.query(query.to_string());
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

static DB: OnceLock<SurrealDB> = OnceLock::new();

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database not initialized. Call init_db() first")]
    DbNotInitialized,
    #[error("Database already initialized")]
    DbAlreadyInitialized,
}
pub fn init_db(config: DbConfig) -> Result<()> {
    let db = SurrealDB::new(config)?;
    DB.set(db).map_err(|_| Error::DbAlreadyInitialized)?;
    Ok(())
}

pub fn db() -> &'static SurrealDB {
    DB.get()
        .expect("Database not initialized. Call init_db() first")
}
