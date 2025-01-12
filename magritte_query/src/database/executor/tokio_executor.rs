use std::sync::Arc;
use std::time::Duration;
use anyhow::{anyhow, bail, Result};
use async_channel::{bounded, Receiver, Sender};
use dashmap::DashMap;
use futures::TryFutureExt;
use serde_json::Value as JsonValue;
use deadpool_surrealdb::{Object, Pool};
use surrealdb::sql::Value;
use thiserror::Error;

#[cfg(feature = "rt-tokio")]
use tokio::sync::Notify;
#[cfg(feature = "rt-tokio")]
use tokio::time::sleep;
#[cfg(feature = "rt-tokio")]
use tokio::task::JoinHandle;
use crate::database::executor::{BaseExecutor, ExecutorConfig, ExecutorMetrics, QueryRequest};
use crate::database::QueryType;
use crate::database::runtime::RuntimeManager;
use crate::database::rw::RwLock;
use crate::{Query};

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 100;

#[derive(Debug, Error)]
enum QueryError {
    #[error("Query error: {0}")]
    GenericError(#[from] anyhow::Error),
    #[error("Query timeout")]
    Timeout,
    #[error("Query execution error: {0}")]
    ExecutionError(#[from] surrealdb::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Retryable error: {0}")]
    RetryableError(Box<QueryError>),
}

/// Event loop that processes queries on a dedicated connection
#[cfg(feature = "rt-tokio")]
struct EventLoop {
    id: usize,
    rx: Receiver<QueryRequest>,
    connection: Object, // Derefs to Surreal<Any>
    metrics: Arc<RwLock<ExecutorMetrics>>,
    notify_shutdown: Arc<Notify>,
    config: ExecutorConfig,
}

#[cfg(feature = "rt-tokio")]
impl EventLoop {
    fn new(
        id: usize,
        rx: Receiver<QueryRequest>,
        connection: Object,
        metrics: Arc<RwLock<ExecutorMetrics>>,
        notify_shutdown: Arc<Notify>,
        config: ExecutorConfig,
    ) -> Self {
        Self {
            id,
            rx,
            connection,
            metrics,
            notify_shutdown,
            config,
        }
    }

    async fn run(&self) -> Result<()> {
        let mut consecutive_errors = 0;

        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = self.notify_shutdown.notified() => {
                    break;
                }
                
                // Process next query
                Ok(request) = self.rx.recv() => {
                    let start = std::time::Instant::now();
                    
                    // Execute query with retries
                    let result = self.execute_with_retries(request.clone()).await;
                    
                    // Update metrics
                    {
                        let mut metrics = self.metrics.write().await;
                        metrics.queries_executed += 1;
                        metrics.avg_query_time = (metrics.avg_query_time * (metrics.queries_executed - 1) as f64 
                            + start.elapsed().as_secs_f64()) / metrics.queries_executed as f64;
                        
                        match &result {
                            Ok(_) => {
                                consecutive_errors = 0;
                            }
                            Err(e) => {
                                metrics.queries_failed += 1;
                                consecutive_errors += 1;
                                
                                // Log error
                                tracing::error!(
                                    "Query execution failed on event loop {}: {}",
                                    self.id, e
                                );
                            }
                        }
                    }
                    
                    // Send response
                    let result = result.map_err(|e| anyhow!(e));
                    if let Err(e) = request.response_tx.send(result).await {
                        tracing::error!("Failed to send query response: {}", e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn execute_with_retries(
        &self,
        request: QueryRequest,
    ) -> Result<JsonValue, QueryError> {
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < MAX_RETRIES {
            match self.execute_query(request.clone()).await {
                Ok(result) => return Ok(result),
                Err(QueryError::RetryableError(e)) => {
                    attempts += 1;
                    last_error = Some(e);
                    
                    if attempts < MAX_RETRIES {
                        sleep(Duration::from_millis(RETRY_DELAY_MS * 2u64.pow(attempts))).await;
                        continue;
                    }
                }
                Err(e) => return Err(e),
            }
        }
        
        Err(QueryError::RetryableError(last_error.unwrap_or_else(|| Box::new(anyhow!("Max retries exceeded").into()))))
    }
    
    async fn execute_query(
        &self,
        request: QueryRequest,
    ) -> Result<JsonValue, QueryError> {
        match request.query_type {
            QueryType::Read => self.execute_read(request).await,
            QueryType::Write => self.execute_write(request).await,
            QueryType::Schema => self.execute_schema(request).await,
        }
    }
    
    async fn execute_read(
        &self,
        request: QueryRequest,
    ) -> Result<JsonValue, QueryError> {
        // Execute query with timeout
        let timeout = self.config.query_timeout;
        
        tokio::select! {
            result = async {
                let mut response = self.connection.query(&request.query)
                    .bind(request.params)
                    .await
                    ?;
                let db_value: Option<Value> = response.take(0)?;
                let value = serde_json::to_value(db_value)?;
                Ok::<JsonValue, QueryError>(value)
            } => {
                result
            }
            _ = sleep(timeout) => {
                Err(QueryError::Timeout)
            }
        }
    }
    
    async fn execute_write(
        &self,
        request: QueryRequest,
    ) -> Result<JsonValue, QueryError> {
        // Use Query::begin() for transaction
        let tx = Query::begin();
        let tx = tx.raw(&request.query);
        
        // Execute query within transaction with timeout
        let timeout = self.config.query_timeout;
        
        let result = tokio::select! {
            result = async {
                let mut response = tx.execute(self.connection.as_ref())
                    .await
                    ?;
                let db_value: Option<Value> = response.take(0)?;
                let value = serde_json::to_value(db_value)?;
                Ok::<JsonValue, QueryError>(value)
            } => {
                result
            }
            _ = sleep(timeout) => {
                Err(QueryError::Timeout)
            }
        };
        
        result
    }
    
    async fn execute_schema(
        &self,
        request: QueryRequest,
    ) -> Result<JsonValue, QueryError> {
        // Schema changes are executed atomically
        self.execute_write(request).await
    }
}

/// Tokio-based executor implementation
#[cfg(feature = "rt-tokio")]
pub struct TokioExecutor {
    config: ExecutorConfig,
    event_loops: DashMap<usize, Sender<QueryRequest>>,
    event_loop_handles: DashMap<usize, JoinHandle<Result<()>>>,
    pool: Pool,
    runtime: Arc<RuntimeManager>,
    metrics: Arc<RwLock<ExecutorMetrics>>,
    notify_shutdown: Arc<Notify>,
}

#[cfg(feature = "rt-tokio")]
#[async_trait::async_trait]
impl BaseExecutor for TokioExecutor {
    async fn run(&self) -> Result<()> {
        // Create event loops
        for i in 0..self.config.max_connections {
            let (tx, rx) = bounded(1000);
            let metrics = self.metrics.clone();
            let connection = self.pool.get().await?;
            let notify = self.notify_shutdown.clone();
            let config = self.config.clone();
            
            // Spawn event loop
            let handle = tokio::spawn(async move {
                let event_loop = EventLoop::new(i, rx, connection, metrics, notify, config);
                event_loop.run().await
            });
            
            self.event_loops.insert(i, tx);
            self.event_loop_handles.insert(i, handle);
            
            // Update metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.idle_connections += 1;
            }
        }
        
        // Wait for all event loops to finish
        for i in 0..self.config.max_connections {
            if let Some((_, handle)) = self.event_loop_handles.remove(&i) {
                if let Err(e) = handle.await? {
                    tracing::error!("Event loop {} failed: {}", i, e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        // Signal all event loops to shut down
        self.notify_shutdown.notify_waiters();
        
        // Wait for all event loops to finish
        for i in 0..self.config.max_connections {
            if let Some((_, handle)) = self.event_loop_handles.remove(&i) {
                if let Err(e) = handle.await? {
                    tracing::error!("Event loop {} failed during shutdown: {}", i, e);
                }
            }
        }
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.active_connections = 0;
            metrics.idle_connections = 0;
        }
        
        Ok(())
    }
    
    async fn metrics(&self) -> ExecutorMetrics {
        self.metrics.read().await.clone()
    }
    
    async fn execute_raw(&self, request: QueryRequest) -> Result<JsonValue> {
        // Get next available event loop using round-robin
        let event_loop_count = self.event_loops.len();
        let event_loop_id = {
            let metrics = self.metrics.read().await;
            metrics.queries_executed % event_loop_count
        };
        
        // Get sender for chosen event loop
        let sender = self.event_loops.get(&event_loop_id)
            .map(|pair| pair.value().clone())
            .ok_or_else(|| anyhow!("Event loop not found"))?;
            
        // Send request to event loop
        sender.send(request)
            .await
            .map_err(|e| anyhow!("Failed to send request to event loop: {}", e))?;
            
        Ok(JsonValue::Null) // Actual response will be sent through response_tx
    }
}

#[cfg(feature = "rt-tokio")]
impl TokioExecutor {
    pub fn new(
        config: ExecutorConfig,
        pool: Pool,
        runtime: Arc<RuntimeManager>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            event_loops: DashMap::new(),
            event_loop_handles: DashMap::new(),
            pool,
            runtime,
            metrics: Arc::new(RwLock::new(ExecutorMetrics::default())),
            notify_shutdown: Arc::new(Notify::new()),
        })
    }
    
    // Ensure all event loops are properly cleaned up on drop
    fn cleanup(&self) {
        for handle in self.event_loop_handles.iter() {
            handle.abort();
        }
    }
}

#[cfg(feature = "rt-tokio")]
impl Drop for TokioExecutor {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(feature = "rt-tokio")]
#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::*;
    use crate::database::executor::tokio_executor::TokioExecutor;
    use crate::database::runtime::{RuntimeConfig, RuntimeManager};
    use crate::database::{DbConfig, QueryType};
    use anyhow::Result;
    use anyhow::{anyhow, bail};
    use async_channel::bounded;
    use deadpool_surrealdb::{Credentials, Pool, Runtime};
    use std::time::Duration;
    use tokio::time::timeout;
    use crate::database::executor::{BaseExecutor, ExecutorConfig, QueryRequest};
    use crate::database::scheduler::QueryPriority;

    const TEST_TIMEOUT: Duration = Duration::from_secs(1);
    const QUERY_TIMEOUT: Duration = Duration::from_millis(500);
    const MAX_CONNECTIONS: u32 = 2;

    struct TestExecutor {
        executor: Arc<TokioExecutor>,
        _pool: Pool, // Keep pool alive for test duration
    }

    impl TestExecutor {
        async fn new() -> Result<Self> {
            let config = ExecutorConfig {
                max_connections: MAX_CONNECTIONS as usize,
                connection_timeout: Duration::from_millis(100),
                query_timeout: QUERY_TIMEOUT,
                use_prepared_statements: false,
            };

            let db_config = DbConfig::builder()
                .namespace("test")
                .database("test")
                .host("mem://")
                .credentials(Credentials::Root {
                    user: "root".to_string(),
                    pass: "root".to_string(),
                });

            let pool = db_config
                .build()
                .map_err(|e| anyhow!(e))?
                .create_pool(Some(Runtime::Tokio1))?;

            let runtime = Arc::new(RuntimeManager::new(RuntimeConfig::default()));
            let executor = Arc::new(TokioExecutor::new(config, pool.clone(), runtime)?);
            executor.run().await?;

            Ok(Self {
                executor,
                _pool: pool,
            })
        }

        async fn cleanup(self) -> Result<()> {
            self.executor.stop().await
        }
    }

    async fn create_test_query(
        query: &str,
        query_type: QueryType,
    ) -> (QueryRequest, async_channel::Receiver<Result<serde_json::Value>>) {
        let (tx, rx) = bounded(1);
        let request = QueryRequest {
            query: query.to_string(),
            params: vec![],
            priority: QueryPriority::Low,
            query_type,
            response_tx: tx,
        };
        (request, rx)
    }

    #[tokio::test]
    async fn test_basic_read_query() -> Result<()> {
        let test = TestExecutor::new().await?;
        let (request, rx) = create_test_query("SELECT * FROM test LIMIT 1", QueryType::Read).await;

        timeout(TEST_TIMEOUT, async {
            test.executor.execute_raw(request).await?;
            let response = rx.recv().await?;
            assert!(response.is_ok());
            test.cleanup().await
        })
            .await?
    }

    #[tokio::test]
    async fn test_concurrent_queries() -> Result<()> {
        let test = TestExecutor::new().await?;
        let mut handles = Vec::new();

        timeout(TEST_TIMEOUT, async {
            for i in 0..5 {
                let (request, _) = create_test_query(
                    &format!("SELECT * FROM test WHERE id = {}", i),
                    QueryType::Read,
                )
                    .await;

                let exec = test.executor.clone();
                handles.push(tokio::spawn(async move {
                    exec.execute_raw(request).await
                }));
            }

            for handle in handles {
                handle.await??;
            }

            test.cleanup().await
        })
            .await?
    }

    #[tokio::test]
    async fn test_query_timeout() -> Result<()> {
        let test = TestExecutor::new().await?;
        let (request, rx) = create_test_query("SELECT sleep(1)", QueryType::Read).await;

        timeout(TEST_TIMEOUT, async {
            test.executor.execute_raw(request).await?;
            let response = rx.recv().await?;
            assert!(matches!(response, Err(e) if e.to_string().contains("timeout")));
            test.cleanup().await
        })
            .await?
    }

    #[tokio::test]
    async fn test_graceful_shutdown() -> Result<()> {
        let test = TestExecutor::new().await?;
        let mut handles = Vec::new();

        timeout(TEST_TIMEOUT, async {
            // Start some queries
            for i in 0..3 {
                let (request, _) = create_test_query(
                    &format!("SELECT sleep(0.{})", i),
                    QueryType::Read
                ).await;

                let exec = test.executor.clone();
                handles.push(tokio::spawn(async move {
                    exec.execute_raw(request).await
                }));
            }

            // Quick sleep to let queries start
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Initiate shutdown
            test.cleanup().await?;

            // Verify queries were handled
            for handle in handles {
                match handle.await {
                    Ok(_) => (), // Completed
                    Err(e) if e.is_cancelled() => (), // Cancelled
                    Err(e) => bail!("Unexpected error: {}", e),
                }
            }

            Ok(())
        })
            .await?
    }

    #[tokio::test]
    async fn test_metrics_collection() -> Result<()> {
        let test = TestExecutor::new().await?;

        timeout(TEST_TIMEOUT, async {
            let start_metrics = test.executor.metrics().await;

            // Execute mix of queries
            let queries = vec![
                ("SELECT * FROM test", QueryType::Read),
                ("INVALID SQL", QueryType::Read),
                ("SELECT * FROM test WHERE id = 1", QueryType::Read),
            ];

            for (query, query_type) in queries {
                let (request, _) = create_test_query(query, query_type).await;
                let _ = test.executor.execute_raw(request).await;
            }

            let end_metrics = test.executor.metrics().await;
            assert!(end_metrics.queries_executed > start_metrics.queries_executed);
            assert!(end_metrics.queries_failed > start_metrics.queries_failed);
            assert!(end_metrics.avg_query_time >= 0.0);

            test.cleanup().await
        })
            .await?
    }
}