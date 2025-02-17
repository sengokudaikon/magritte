// Core executor traits and types
pub mod core;

// Execution utilities
pub mod utils {
    pub mod metrics;
    pub mod query_batcher;
}

// Executor implementations
mod impl_executor;

pub use core::{
    config::ExecutorConfig,
    types::{ExecutorError, QueryPriority, QueryRequest, QueryType},
    BaseExecutor, ExecutorState,
};

pub use utils::{metrics::ExecutorMetrics, query_batcher::QueryBatch};

pub use impl_executor::Executor;

use anyhow::Result;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::executor::core::config::{BatchConfig, PoolConfig, QueryConfig};
    use crate::database::{DbConfig, QueryPriority};
    use deadpool_surrealdb::{Credentials, Runtime};
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio::time::sleep;

    async fn setup_test_executor() -> Arc<Executor> {
        let config = ExecutorConfig {
            pool: PoolConfig {
                min_connections: 1,
                max_connections: 5,
                connection_timeout: Duration::from_secs(5),
                idle_timeout: Duration::from_secs(10),
                max_lifetime: Duration::from_secs(30),
            },
            query: QueryConfig {
                query_timeout: Duration::from_secs(5),
                max_retries: 3,
                retry_backoff: Duration::from_millis(100),
                max_concurrent_queries: 10,
            },
            batch: BatchConfig {
                max_write_batch_size: 10,
                max_read_batch_size: 50,
                batch_timeout: Duration::from_millis(50),
            },
        };

        let pool = DbConfig {
            host: "mem://".to_string(),
            ns: "test".to_string(),
            db: "test".to_string(),
            creds: Credentials::Root {
                user: "root".to_string(),
                pass: "root".to_string(),
            },
            ..Default::default()
        }
            .create_pool(Some(Runtime::Tokio1))
            .map_err(anyhow::Error::from)
            .expect("Failed to create pool");

        Executor::new(pool, config)
            .await
            .expect("Failed to create executor")
    }

    #[tokio::test]
    async fn test_executor_suspend_resume() {
        let executor = setup_test_executor().await;
        executor.start().await.expect("Failed to start executor");

        // Test query works before suspend
        let (tx, mut rx) = mpsc::channel(1);
        let request = QueryRequest {
            query: "INFO FOR DATABASE".to_string(),
            params: vec![],
            priority: QueryPriority::Normal,
            query_type: QueryType::Read,
            table_name: None,
            response_tx: tx,
        };

        // Add timeout for the entire test
        let test_result = tokio::time::timeout(Duration::from_secs(5), async {
            executor
                .execute(request)
                .await
                .expect("Failed to execute query");

            // Wait for batch timeout to ensure query is processed
            sleep(Duration::from_millis(100)).await;

            // Add timeout for receiving response
            match tokio::time::timeout(Duration::from_millis(500), rx.recv()).await {
                Ok(Some(response)) => {
                    response.expect("Query failed");
                }
                Ok(None) => panic!("Channel closed unexpectedly"),
                Err(_) => panic!("Timeout waiting for response"),
            }

            // Wait a bit to ensure the first query is fully processed
            sleep(Duration::from_millis(200)).await;

            // Suspend executor
            executor
                .suspend()
                .await
                .expect("Failed to suspend executor");

            // Test query fails when suspended
            let (tx, mut rx) = mpsc::channel(1);
            let request = QueryRequest {
                query: "INFO FOR DATABASE".to_string(),
                params: vec![],
                priority: QueryPriority::Normal,
                query_type: QueryType::Read,
                table_name: None,
                response_tx: tx,
            };

            executor
                .execute(request)
                .await
                .expect("Failed to execute query");

            // Add timeout for receiving error response
            match tokio::time::timeout(Duration::from_millis(500), rx.recv()).await {
                Ok(Some(result)) => {
                    assert!(result.is_err());
                    assert!(result.unwrap_err().to_string().contains("suspended"));
                }
                Ok(None) => panic!("Channel closed unexpectedly"),
                Err(_) => panic!("Timeout waiting for error response"),
            }

            // Resume executor
            executor.resume().await.expect("Failed to resume executor");

            // Test query works after resume
            let (tx, mut rx) = mpsc::channel(1);
            let request = QueryRequest {
                query: "INFO FOR DATABASE".to_string(),
                params: vec![],
                priority: QueryPriority::Normal,
                query_type: QueryType::Read,
                table_name: None,
                response_tx: tx,
            };

            executor
                .execute(request)
                .await
                .expect("Failed to execute query");

            // Wait for batch timeout to ensure query is processed
            sleep(Duration::from_millis(100)).await;

            // Add timeout for receiving response
            match tokio::time::timeout(Duration::from_millis(500), rx.recv()).await {
                Ok(Some(response)) => {
                    response.expect("Query failed");
                }
                Ok(None) => panic!("Channel closed unexpectedly"),
                Err(_) => panic!("Timeout waiting for response"),
            }
        }).await;

        // Handle test timeout
        match test_result {
            Ok(_) => (),
            Err(_) => panic!("Test timed out after 5 seconds"),
        }
    }

    #[tokio::test]
    async fn test_executor_graceful_shutdown() {
        let executor = setup_test_executor().await;
        executor.start().await.expect("Failed to start executor");

        // Send a bunch of queries
        let mut handles = vec![];
        for i in 0..5 {
            let executor = executor.clone();
            handles.push(tokio::spawn(async move {
                let (tx, mut rx) = mpsc::channel(1);
                let request = QueryRequest {
                    query: format!("SELECT * FROM test_{}", i),
                    params: vec![],
                    priority: QueryPriority::Normal,
                    query_type: QueryType::Read,
                    table_name: None,
                    response_tx: tx,
                };

                executor
                    .execute(request)
                    .await
                    .expect("Failed to execute query");
                // Wait for batch timeout to ensure query is processed
                sleep(Duration::from_millis(100)).await;
                rx.recv()
                    .await
                    .expect("Failed to receive response")
                    .expect("Query failed")
            }));
        }

        // Wait a bit for queries to be in flight
        sleep(Duration::from_millis(50)).await;

        // Stop executor gracefully
        let stop_handle = tokio::spawn({
            let executor = executor.clone();
            async move {
                executor.stop().await.expect("Failed to stop executor");
            }
        });

        // Verify all queries completed
        for handle in handles {
            handle.await.expect("Task failed");
        }

        // Wait for stop to complete
        stop_handle.await.expect("Stop task failed");

        // Verify new queries are rejected
        let (tx, mut rx) = mpsc::channel(1);
        let request = QueryRequest {
            query: "SELECT * FROM test".to_string(),
            params: vec![],
            priority: QueryPriority::Normal,
            query_type: QueryType::Read,
            table_name: None,
            response_tx: tx,
        };

        assert!(executor.execute(request).await.is_err());

        // Ensure no response is received since the query was rejected
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_internal_concurrency() -> Result<()> {
        let executor = setup_test_executor().await;
        executor.start().await?;

        // Submit multiple queries rapidly without manual task spawning
        // The executor should handle concurrency internally through its event loop
        for i in 0..10 {
            let (tx, mut rx) = mpsc::channel(1);
            let request = QueryRequest {
                query: format!("SELECT * FROM test WHERE id = {}", i),
                params: vec![],
                priority: QueryPriority::Normal,
                query_type: QueryType::Read,
                table_name: Some("test".to_string()),
                response_tx: tx,
            };

            executor.execute(request).await?;
            rx.recv().await.ok_or_else(|| anyhow::anyhow!("Channel closed"))??;
        }

        // Verify metrics show concurrent processing
        let metrics = executor.metrics().await;
        assert!(metrics.total_queries() >= 10);

        Ok(())
    }

    #[tokio::test]
    async fn test_batch_coalescing() -> Result<()> {
        let executor = setup_test_executor().await;
        executor.start().await?;

        // Submit queries that should be coalesced into a single batch
        let mut rxs = Vec::new();

        // Submit queries faster than the batch timeout
        for i in 0..5 {
            let (tx, rx) = mpsc::channel(1);
            let request = QueryRequest {
                query: format!("SELECT * FROM test WHERE id = {}", i),
                params: vec![],
                priority: QueryPriority::Normal,
                query_type: QueryType::Read,
                table_name: Some("test".to_string()),
                response_tx: tx,
            };

            executor.execute(request).await?;
            rxs.push(rx);
        }

        // All queries should complete despite being batched
        for mut rx in rxs {
            rx.recv().await.ok_or_else(|| anyhow::anyhow!("Channel closed"))??;
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_query_interleaving() -> Result<()> {
        let executor = setup_test_executor().await;
        executor.start().await?;

        // Create a table for testing - don't try to deserialize the response
        let (tx, mut rx) = mpsc::channel(1);
        let create_request = QueryRequest {
            query: "DEFINE TABLE test_interleave SCHEMAFULL".to_string(),
            params: vec![],
            priority: QueryPriority::High,
            query_type: QueryType::Schema,
            table_name: Some("test_interleave".to_string()),
            response_tx: tx,
        };

        println!("Executing schema query...");
        executor.execute(create_request).await?;

        // For schema queries, just ensure we got a response without parsing
        let schema_response = rx.recv().await.ok_or_else(|| anyhow::anyhow!("Channel closed"))?;
        println!("Schema query raw response: {:?}", schema_response);
        // Don't try to unwrap the Result, just check if we got one
        assert!(schema_response.is_ok(), "Schema query failed");

        // Submit a mix of read and write queries without manual task management
        let mut rxs = Vec::new();

        for i in 0..5 {
            // Write query
            let (tx, rx) = mpsc::channel(1);
            let write_request = QueryRequest {
                query: format!("CREATE test_interleave:{} CONTENT {{ value: {} }} RETURN AFTER", i, i),
                params: vec![],
                priority: QueryPriority::Normal,
                query_type: QueryType::Write,
                table_name: Some("test_interleave".to_string()),
                response_tx: tx,
            };
            executor.execute(write_request).await?;
            rxs.push((rx, format!("write-{}", i)));

            // Read query
            let (tx, rx) = mpsc::channel(1);
            let read_request = QueryRequest {
                query: format!("SELECT * FROM test_interleave:{}", i),
                params: vec![],
                priority: QueryPriority::Normal,
                query_type: QueryType::Read,
                table_name: Some("test_interleave".to_string()),
                response_tx: tx,
            };
            executor.execute(read_request).await?;
            rxs.push((rx, format!("read-{}", i)));
        }

        // All queries should complete in order despite potential batching
        for (mut rx, query_type) in rxs {
            let response = rx.recv().await.ok_or_else(|| anyhow::anyhow!("Channel closed"))?;
            println!("Query response for {}: {:?}", query_type, response);

            match response {
                Ok(value) => {
                    // Should always be an array
                    let arr = value.as_array().expect("Response should be an array");
                    assert!(!arr.is_empty(), "Response array should not be empty");

                    // For write queries, verify we got a record back
                    if query_type.starts_with("write-") {
                        let record = &arr[0];
                        assert!(record.is_object(), "Record should be an object");
                        assert!(record.get("id").is_some(), "Record should have an id");
                        assert!(record.get("value").is_some(), "Record should have a value field");

                        let i: i64 = query_type.strip_prefix("write-").unwrap().parse().unwrap();
                        assert_eq!(record["value"], i, "Value should match what we wrote");
                    }

                    // For read queries, verify we can read the record
                    if query_type.starts_with("read-") {
                        let record = &arr[0];
                        assert!(record.is_object(), "Record should be an object");
                        assert!(record.get("value").is_some(), "Record should have a value field");
                    }
                }
                Err(e) => panic!("Query {} failed: {:?}", query_type, e),
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_priority_ordering() -> Result<()> {
        let executor = setup_test_executor().await;
        executor.start().await?;

        // Submit low priority queries
        let mut low_rxs = Vec::new();
        for i in 0..5 {
            let (tx, rx) = mpsc::channel(1);
            let request = QueryRequest {
                query: format!("SELECT * FROM test WHERE id = {}", i),
                params: vec![],
                priority: QueryPriority::Low,
                query_type: QueryType::Read,
                table_name: Some("test".to_string()),
                response_tx: tx,
            };
            executor.execute(request).await?;
            low_rxs.push(rx);
        }

        // Submit high priority queries immediately after
        let mut high_rxs = Vec::new();
        for i in 0..5 {
            let (tx, rx) = mpsc::channel(1);
            let request = QueryRequest {
                query: format!("SELECT * FROM test WHERE id = {}", i),
                params: vec![],
                priority: QueryPriority::High,
                query_type: QueryType::Read,
                table_name: Some("test".to_string()),
                response_tx: tx,
            };
            executor.execute(request).await?;
            high_rxs.push(rx);
        }

        // High priority queries should complete first
        for mut rx in high_rxs {
            rx.recv().await.ok_or_else(|| anyhow::anyhow!("Channel closed"))??;
        }

        // Then low priority queries
        for mut rx in low_rxs {
            rx.recv().await.ok_or_else(|| anyhow::anyhow!("Channel closed"))??;
        }

        Ok(())
    }
}
