pub mod executor;
use anyhow::{anyhow, Result};
pub(crate) use deadpool_surrealdb::Config as DbConfig;
use deadpool_surrealdb::Runtime;
use executor::core::config::{BatchConfig, PoolConfig, QueryConfig};
use executor::{BaseExecutor, Executor, ExecutorConfig, ExecutorMetrics};
pub use executor::{QueryPriority, QueryRequest, QueryType};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::channel;

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
            .execute(request)
            .await
            .map_err(anyhow::Error::from)?;

        // Wait for response from channel
        let result = rx.recv().await.ok_or_else(|| anyhow!("Channel closed"))??;

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

    /// Get current executor metrics (internal use)
    pub(crate) async fn metrics(&self) -> Result<Arc<ExecutorMetrics>> {
        Ok(self.executor.metrics().await)
    }
}

unsafe impl Send for SurrealDB {}

unsafe impl Sync for SurrealDB {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::time::Duration;
    use tokio::time::sleep;

    async fn setup_test_db() -> Arc<SurrealDB> {
        let config = DbConfig {
            host: "mem://".to_string(),
            ns: "test".to_string(),
            db: "test".to_string(),
            max_connections: 5,
            connect_timeout: 5,
            idle_timeout: 10,
            ..Default::default()
        };

        SurrealDB::new(config).await.expect("Failed to create database")
    }

    // #[tokio::test]
    // async fn test_db_suspend_resume() -> Result<()> {
    //     let db = setup_test_db().await;
    //
    //     // Test query works before suspend
    //     let result: Vec<Value> = db.execute_raw("INFO FOR DATABASE".to_string()).await?;
    //     assert!(!result.is_empty());
    //
    //     // Suspend executor
    //     db.executor.suspend().await?;
    //     sleep(Duration::from_millis(100)).await;
    //
    //     // Test query fails when suspended
    //     let result = db.execute_raw::<Value>("INFO FOR DATABASE".to_string()).await;
    //     assert!(result.is_err());
    //     assert!(result.unwrap_err().to_string().contains("suspended"));
    //
    //     // Resume executor
    //     db.executor.resume().await?;
    //     sleep(Duration::from_millis(100)).await;
    //
    //     // Test query works after resume
    //     let result: Vec<Value> = db.execute_raw("INFO FOR DATABASE".to_string()).await?;
    //     assert!(!result.is_empty());
    //
    //     Ok(())
    // }
    //
    // #[tokio::test]
    // async fn test_db_graceful_shutdown() -> Result<()> {
    //     let db = setup_test_db().await;
    //
    //     // Send a bunch of queries
    //     let mut handles = vec![];
    //     for i in 0..5 {
    //         let db = db.clone();
    //         handles.push(tokio::spawn(async move {
    //             let result: Vec<Value> = db
    //                 .execute_raw(format!("SELECT * FROM test_{}", i))
    //                 .await
    //                 .expect("Failed to execute query");
    //             assert!(result.is_empty()); // No data exists yet
    //         }));
    //     }
    //
    //     // Wait a bit for queries to be in flight
    //     sleep(Duration::from_millis(50)).await;
    //
    //     // Stop executor gracefully
    //     db.executor.stop().await?;
    //
    //     // Verify all queries completed
    //     for handle in handles {
    //         handle.await.expect("Task failed");
    //     }
    //
    //     // Verify new queries are rejected
    //     let result = db.execute_raw::<Value>("SELECT * FROM test".to_string()).await;
    //     assert!(result.is_err());
    //     assert!(result.unwrap_err().to_string().contains("not running"));
    //
    //     Ok(())
    // }
    //
    // #[tokio::test]
    // async fn test_db_query_types() -> Result<()> {
    //     let db = setup_test_db().await;
    //
    //     // Test schema query using execute_raw since we don't care about the return type
    //     let schema_result = db.execute::<Value>(
    //         "DEFINE TABLE test SCHEMAFULL".to_string(),
    //         vec![],
    //         QueryType::Schema,
    //         Some("test".to_string()),
    //     ).await;
    //     match &schema_result {
    //         Ok(values) => println!("Schema query response: {:?}", values),
    //         Err(e) => println!("Schema query error: {:?}", e),
    //     }
    //     schema_result?;
    //
    //     // Test write query with logging
    //     let write_result = db
    //         .execute::<Value>(
    //             "CREATE test:1 CONTENT { value: 'test', username: 'john-smith' } RETURN AFTER".to_string(),
    //             vec![],
    //             QueryType::Write,
    //             Some("test".to_string()),
    //         )
    //         .await;
    //     match &write_result {
    //         Ok(values) => println!("Write query response: {:?}", values),
    //         Err(e) => println!("Write query error: {:?}", e),
    //     }
    //     let result = write_result?;
    //     assert!(!result.is_empty());
    //     let first_record = &result[0];
    //     assert_eq!(first_record["value"], "test");
    //     assert_eq!(first_record["username"], "john-smith");
    //
    //     // Test read query with logging
    //     let read_result = db
    //         .execute::<Value>(
    //             "SELECT * FROM test WHERE value = 'test'".to_string(),
    //             vec![],
    //             QueryType::Read,
    //             Some("test".to_string()),
    //         )
    //         .await;
    //     match &read_result {
    //         Ok(values) => println!("Read query response: {:?}", values),
    //         Err(e) => println!("Read query error: {:?}", e),
    //     }
    //     let result = read_result?;
    //     assert!(!result.is_empty());
    //     let first_record = &result[0];
    //     assert_eq!(first_record["value"], "test");
    //     assert_eq!(first_record["username"], "john-smith");
    //
    //     Ok(())
    // }
}
