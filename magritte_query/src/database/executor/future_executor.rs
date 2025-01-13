use crate::database::executor::{BaseExecutor, ExecutorConfig, ExecutorMetrics, QueryRequest};
use crate::database::runtime::RuntimeManager;
use crate::database::QueryType;
use crate::Query;
use anyhow::{anyhow, Result};
use async_channel::{bounded, Receiver, Sender};
use dashmap::DashMap;
use deadpool_surrealdb::Pool;
use futures_concurrency::prelude::*;
use futures_locks::RwLock;
use futures_timer::Delay;
use futures_util::StreamExt;
use futures_util::future::{FutureExt, select};
use serde_json::{Value as JsonValue, Value};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

const CHANNEL_SIZE: usize = 1000;
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 100;

#[cfg(test)]
const TEST_MAX_ITERATIONS: usize = 5;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Query execution failed: {0}")]
    ExecutionError(String),
    #[error("Query timed out")]
    Timeout,
    #[error("Connection error: {0}")]
    ConnectionError(String),
}

#[derive(Debug, Default)]
struct Metrics {
    active_connections: AtomicUsize,
    idle_connections: AtomicUsize,
    queries_executed: AtomicUsize,
    queries_failed: AtomicUsize,
    total_query_time: AtomicUsize,
}

#[derive(Clone)]
pub struct FutureExecutor {
    config: ExecutorConfig,
    pool: Pool,
    runtime: Arc<RuntimeManager>,
    metrics: Arc<Metrics>,
    request_tx: async_channel::Sender<QueryRequest>,
    request_rx: async_channel::Receiver<QueryRequest>,
    running: Arc<AtomicBool>,
    state: Arc<RwLock<ExecutorState>>,
    event_loops: DashMap<usize, Sender<QueryRequest>>,
    #[cfg(test)]
    test_mode: bool,
    #[cfg(test)]
    iteration_count: Arc<AtomicUsize>,
}

#[derive(Default)]
struct ExecutorState {
    current_batch: Vec<QueryRequest>,
    last_batch_time: Option<std::time::Instant>,
}

impl FutureExecutor {
    pub fn new(config: ExecutorConfig, pool: Pool, runtime: Arc<RuntimeManager>) -> Result<Self> {
        let (request_tx, request_rx) = bounded(config.max_connections);
        Ok(Self {
            config,
            pool,
            runtime,
            metrics: Arc::new(Metrics::default()),
            request_tx,
            request_rx,
            running: Arc::new(AtomicBool::new(false)),
            state: Arc::new(RwLock::new(ExecutorState::default())),
            event_loops: DashMap::new(),
            #[cfg(test)]
            test_mode: false,
            #[cfg(test)]
            iteration_count: Arc::new(AtomicUsize::new(0)),
        })
    }

    #[cfg(test)]
    pub fn enable_test_mode(self: &Arc<Self>) {
        // SAFETY: This is only called in tests before the executor starts
        unsafe {
            let self_mut = &mut *(Arc::as_ptr(self) as *mut Self);
            self_mut.test_mode = true;
        }
    }

    async fn run_event_loop(
        &self,
        request_rx: async_channel::Receiver<QueryRequest>,
    ) -> Result<()> {
        self.running.store(true, Ordering::SeqCst);

        let max_concurrent = self.config.max_connections;
        let mut active_tasks = HashSet::new();

        while self.running.load(Ordering::SeqCst) {
            #[cfg(test)]
            if self.test_mode {
                let count = self.iteration_count.fetch_add(1, Ordering::SeqCst);
                if count >= TEST_MAX_ITERATIONS {
                    break;
                }
            }

            // Clean up completed tasks
            active_tasks.retain(|task_id| {
                self.metrics.active_connections.load(Ordering::Relaxed) > *task_id
            });

            if active_tasks.len() >= max_concurrent {
                Delay::new(Duration::from_millis(10)).await;
                continue;
            }

            let timeout = Box::pin(Delay::new(Duration::from_millis(100)));
            let request_future = Box::pin(request_rx.recv().fuse());

            match select(request_future, timeout).await {
                futures_util::future::Either::Left((Ok(request), _)) => {
                    let task_id = self
                        .metrics
                        .active_connections
                        .fetch_add(1, Ordering::Relaxed);
                    active_tasks.insert(task_id);

                    let executor = self.clone();
                    let start_time = std::time::Instant::now();

                    // Process request with proper error handling
                    let result = match request.query_type {
                        QueryType::Read => {
                            if let Some(batch) = executor.try_batch_reads(&request).await {
                                let results = executor.execute_parallel_reads(batch).await?;
                                Ok(results[0].clone())
                            } else {
                                executor.execute_read_query(&request).await
                            }
                        }
                        QueryType::Write => {
                            if let Some(table_name) = &request.table_name {
                                if let Some(batch) =
                                    executor.try_batch_writes(&request, table_name).await
                                {
                                    let results = executor
                                        .execute_batched_writes(batch, table_name.clone())
                                        .await?;
                                    Ok(results[0].clone())
                                } else {
                                    executor.execute_write_query(&request).await
                                }
                            } else {
                                executor.execute_write_query(&request).await
                            }
                        }
                        QueryType::Schema => executor.execute_schema_query(&request).await,
                    };

                    executor
                        .handle_query_result(result, start_time, &request)
                        .await;
                    self.metrics
                        .active_connections
                        .fetch_sub(1, Ordering::Relaxed);
                }
                futures_util::future::Either::Left((Err(_), _)) => break, // Channel closed
                futures_util::future::Either::Right(_) => continue,       // Timeout
            }
        }

        // Wait for all active tasks to complete
        while self.metrics.active_connections.load(Ordering::Relaxed) > 0 {
            Delay::new(Duration::from_millis(10)).await;
        }

        Ok(())
    }

    async fn handle_query_result(
        &self,
        result: Result<JsonValue, QueryError>,
        start_time: std::time::Instant,
        request: &QueryRequest,
    ) {
        let elapsed = start_time.elapsed();

        match result {
            Ok(value) => {
                self.metrics
                    .queries_executed
                    .fetch_add(1, Ordering::Relaxed);
                let elapsed_micros = elapsed.as_micros() as usize;
                self.metrics
                    .total_query_time
                    .fetch_add(elapsed_micros, Ordering::Relaxed);

                if let Err(e) = request.response_tx.send(Ok(value)).await {
                    tracing::error!("Failed to send query result: {}", e);
                }
            }
            Err(e) => {
                self.metrics.queries_failed.fetch_add(1, Ordering::Relaxed);
                let error = anyhow!("Query failed: {}", e);
                if let Err(send_err) = request.response_tx.send(Err(error)).await {
                    tracing::error!("Failed to send query error: {}", send_err);
                }
            }
        }
    }

    async fn execute_parallel_reads(&self, requests: Vec<QueryRequest>) -> Result<Vec<JsonValue>, QueryError> {
        let futures = requests
            .into_iter()
            .map(|request| {
                let request = request.clone();
                async move { 
                    self.execute_read_query(&request).await
                }
            })
            .collect::<Vec<_>>();

        // Use futures_concurrency's join for parallel execution
        let results = futures.join().await;
        
        // Convert results into a Vec, propagating errors
        results.into_iter().collect()
    }

    async fn execute_batched_writes(
        &self,
        requests: Vec<QueryRequest>,
        table_name: String,
    ) -> Result<Vec<JsonValue>, QueryError> {
        self.metrics
            .active_connections
            .fetch_add(1, Ordering::Relaxed);
        self.metrics
            .idle_connections
            .fetch_sub(1, Ordering::Relaxed);

        let connection = self
            .pool
            .get()
            .await
            .map_err(|e| QueryError::ConnectionError(e.to_string()))?;

        let mut transaction = Query::begin();

        for request in &requests {
            transaction = transaction.raw(&request.query);
        }

        let transaction = transaction.commit().build();

        let mut response = connection
            .query(transaction)
            .bind(
                requests
                    .iter()
                    .flat_map(|r| r.params.clone())
                    .collect::<Vec<_>>(),
            )
            .await
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        self.metrics
            .active_connections
            .fetch_sub(1, Ordering::Relaxed);
        self.metrics
            .idle_connections
            .fetch_add(1, Ordering::Relaxed);

        let mut results = Vec::with_capacity(requests.len());
        for i in 0..requests.len() {
            let value = response
                .take::<Option<surrealdb::sql::Value>>(i)
                .map_err(|e| QueryError::ExecutionError(e.to_string()))?
                .ok_or_else(|| QueryError::ExecutionError("No value returned".to_string()))?;

            let json_value = serde_json::to_value(value)
                .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

            results.push(json_value);
        }

        Ok(results)
    }

    async fn execute_query(&self, request: &QueryRequest) -> Result<JsonValue, QueryError> {
        match request.query_type {
            QueryType::Read => {
                if let Some(batch) = self.try_batch_reads(request).await {
                    let results = self.execute_parallel_reads(batch).await?;
                    Ok(results[0].clone())
                } else {
                    self.execute_read_query(request).await
                }
            }
            QueryType::Write => {
                if let Some(table_name) = &request.table_name {
                    if let Some(batch) = self.try_batch_writes(request, table_name).await {
                        let results = self
                            .execute_batched_writes(batch, table_name.clone())
                            .await?;
                        Ok(results[0].clone())
                    } else {
                        self.execute_write_query(request).await
                    }
                } else {
                    self.execute_write_query(request).await
                }
            }
            QueryType::Schema => self.execute_schema_query(request).await,
        }
    }

    async fn try_batch_reads(&self, request: &QueryRequest) -> Option<Vec<QueryRequest>> {
        let mut batch = vec![request.clone()];

        while batch.len() < self.config.max_batch_size {
            match self.request_rx.try_recv() {
                Ok(req) if req.query_type == QueryType::Read => {
                    batch.push(req);
                }
                _ => break,
            }
        }

        if batch.len() > 1 {
            Some(batch)
        } else {
            None
        }
    }

    async fn try_batch_writes(
        &self,
        request: &QueryRequest,
        table_name: &str,
    ) -> Option<Vec<QueryRequest>> {
        let mut batch = vec![request.clone()];
        let batch_timeout = Duration::from_millis(self.config.batch_timeout_ms);
        let batch_start = std::time::Instant::now();

        while batch.len() < self.config.max_batch_size && batch_start.elapsed() < batch_timeout {
            match self.request_rx.try_recv() {
                Ok(req) => {
                    if req.query_type == QueryType::Write
                        && req.table_name.as_deref() == Some(table_name)
                    {
                        batch.push(req);
                    } else {
                        if let Err(e) = self.request_tx.try_send(req) {
                            tracing::error!("Failed to put back non-matching request: {}", e);
                        }
                        break;
                    }
                }
                _ => break,
            }
        }

        if batch.len() > 1 {
            Some(batch)
        } else {
            None
        }
    }

    async fn execute_with_retries(&self, request: QueryRequest) -> Result<JsonValue, QueryError> {
        let mut attempts = 0;
        loop {
            let state = self.state.write().await;
            match self.execute_query(&request).await {
                Ok(value) => return Ok(value),
                Err(e) => {
                    attempts += 1;
                    if attempts >= MAX_RETRIES {
                        return Err(e);
                    }
                    drop(state); // Release the lock before delay
                    Delay::new(Duration::from_millis(RETRY_DELAY_MS * 2u64.pow(attempts))).await;
                }
            }
        }
    }

    async fn execute_read_query(&self, request: &QueryRequest) -> Result<JsonValue, QueryError> {
        self.metrics
            .active_connections
            .fetch_add(1, Ordering::Relaxed);
        self.metrics
            .idle_connections
            .fetch_sub(1, Ordering::Relaxed);

        let connection = self
            .pool
            .get()
            .await
            .map_err(|e| QueryError::ConnectionError(e.to_string()))?;

        let mut response = connection
            .query(request.query.as_str())
            .bind(request.params.clone())
            .await
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        self.metrics
            .active_connections
            .fetch_sub(1, Ordering::Relaxed);
        self.metrics
            .idle_connections
            .fetch_add(1, Ordering::Relaxed);

        let value = response
            .take::<Option<surrealdb::Value>>(0)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?
            .ok_or_else(|| QueryError::ExecutionError("No value returned".to_string()))?;

        serde_json::to_value(value).map_err(|e| QueryError::ExecutionError(e.to_string()))
    }

    async fn execute_write_query(&self, request: &QueryRequest) -> Result<JsonValue, QueryError> {
        self.metrics
            .active_connections
            .fetch_add(1, Ordering::Relaxed);
        self.metrics
            .idle_connections
            .fetch_sub(1, Ordering::Relaxed);

        let connection = self
            .pool
            .get()
            .await
            .map_err(|e| QueryError::ConnectionError(e.to_string()))?;

        // Build transaction query based on table name
        let transaction = if let Some(table) = &request.table_name {
            // Add a comment and the query to the transaction
            let mut stmt = Query::begin();
            stmt = stmt.raw(&request.query);
            stmt.commit().build()
        } else {
            // If no table name, just execute the query directly
            request.query.clone()
        };

        let mut response = connection
            .query(transaction)
            .bind(request.params.clone())
            .await
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        self.metrics
            .active_connections
            .fetch_sub(1, Ordering::Relaxed);
        self.metrics
            .idle_connections
            .fetch_add(1, Ordering::Relaxed);

        let value = response
            .take::<Option<surrealdb::sql::Value>>(0)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?
            .ok_or_else(|| QueryError::ExecutionError("No value returned".to_string()))?;

        serde_json::to_value(value).map_err(|e| QueryError::ExecutionError(e.to_string()))
    }

    async fn execute_schema_query(&self, request: &QueryRequest) -> Result<JsonValue, QueryError> {
        // Schema operations need exclusive access
        let _state = self.state.write().await;

        // Wait for all active operations to complete
        while self.metrics.active_connections.load(Ordering::Relaxed) > 1 {
            Delay::new(Duration::from_millis(50)).await;
        }

        // Execute schema operation in isolation
        self.execute_write_query(request).await
    }
}

#[async_trait::async_trait]
impl BaseExecutor for FutureExecutor {
    async fn run(&self) -> Result<()> {
        let request_rx = self.request_rx.clone();
        self.run_event_loop(request_rx).await
    }

    async fn stop(&self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        drop(self.request_tx.clone());

        while self.metrics.active_connections.load(Ordering::Relaxed) > 0 {
            Delay::new(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    async fn metrics(&self) -> Arc<ExecutorMetrics> {
        Arc::new(ExecutorMetrics {
            active_connections: AtomicUsize::new(
                self.metrics.active_connections.load(Ordering::Relaxed),
            ),
            idle_connections: AtomicUsize::new(
                self.metrics.idle_connections.load(Ordering::Relaxed),
            ),
            queries_executed: AtomicUsize::new(
                self.metrics.queries_executed.load(Ordering::Relaxed),
            ),
            queries_failed: AtomicUsize::new(self.metrics.queries_failed.load(Ordering::Relaxed)),
            total_query_time: AtomicUsize::new(
                self.metrics.total_query_time.load(Ordering::Relaxed),
            ),
        })
    }

    async fn execute_raw(&self, request: QueryRequest) -> Result<Value> {
        let (response_tx, response_rx) = bounded(1);
        let request = QueryRequest {
            response_tx,
            ..request
        };

        self.request_tx
            .send(request)
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        response_rx
            .recv()
            .await
            .map_err(|e| anyhow!("Failed to receive response: {}", e))?
            .map_err(|e| anyhow!("Query failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::runtime::{RuntimeConfig, RuntimeManager};
    use crate::database::scheduler::QueryPriority;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;
    use deadpool_surrealdb::Runtime;
    use crate::database::DbConfig;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestRecord {
        id: String,
        name: String,
        value: i32,
    }

    async fn setup_executor() -> Result<Arc<FutureExecutor>> {
        let config = DbConfig {
            max_connections: 10,
            connect_timeout: 30,
            idle_timeout: 30,
            ..Default::default()
        };

        let pool = config
            .create_pool(Some(Runtime::Tokio1))
            .map_err(anyhow::Error::from)?;

        let executor_config = ExecutorConfig {
            max_connections: config.max_connections as usize,
            connection_timeout: Duration::from_secs(config.connect_timeout),
            query_timeout: Duration::from_secs(config.idle_timeout),
            use_prepared_statements: true,
            max_batch_size: 100,
            batch_timeout_ms: 50,
        };

        let executor = FutureExecutor::new(
            executor_config,
            pool,
            Arc::new(RuntimeManager::new(RuntimeConfig::default())),
        )?;

        Ok(Arc::new(executor))
    }

    #[tokio::test]
    async fn test_parallel_reads() -> Result<()> {
        let executor = setup_executor().await?;
        executor.enable_test_mode();

        // Create test data
        let create_queries = vec![
            "CREATE test:1 SET name = 'test1', value = 1",
            "CREATE test:2 SET name = 'test2', value = 2",
            "CREATE test:3 SET name = 'test3', value = 3",
        ];

        for query in create_queries {
            let (tx, _rx) = bounded(1);
            let request = QueryRequest {
                query: query.to_string(),
                params: vec![],
                priority: QueryPriority::Normal,
                query_type: QueryType::Write,
                table_name: Some("test".to_string()),
                response_tx: tx,
            };
            executor.execute_raw(request).await?;
        }

        // Test parallel reads
        let read_queries: Vec<_> = (1..=3)
            .map(|i| {
                let (tx, rx) = bounded(1);
                let request = QueryRequest {
                    query: format!("SELECT * FROM test:{}", i),
                    params: vec![],
                    priority: QueryPriority::Normal,
                    query_type: QueryType::Read,
                    table_name: Some("test".to_string()),
                    response_tx: tx,
                };
                (request, rx)
            })
            .collect();

        let requests: Vec<_> = read_queries.iter().map(|(req, _)| req.clone()).collect();
        let results = executor.execute_parallel_reads(requests).await?;

        assert_eq!(results.len(), 3);
        for (i, result) in results.iter().enumerate() {
            let record: TestRecord = serde_json::from_value(result.clone())?;
            assert_eq!(record.id, format!("test:{}", i + 1));
            assert_eq!(record.name, format!("test{}", i + 1));
            assert_eq!(record.value, (i + 1) as i32);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_batched_writes() -> Result<()> {
        let executor = setup_executor().await?;
        executor.enable_test_mode();

        // Create batch of write requests
        let write_queries: Vec<_> = (1..=3)
            .map(|i| {
                let (tx, _rx) = bounded(1);
                QueryRequest {
                    query: format!("CREATE test:{} SET name = 'test{}', value = {}", i, i, i),
                    params: vec![],
                    priority: QueryPriority::Normal,
                    query_type: QueryType::Write,
                    table_name: Some("test".to_string()),
                    response_tx: tx,
                }
            })
            .collect();

        let results = executor
            .execute_batched_writes(write_queries, "test".to_string())
            .await?;

        assert_eq!(results.len(), 3);
        for (i, result) in results.iter().enumerate() {
            let record: TestRecord = serde_json::from_value(result.clone())?;
            assert_eq!(record.id, format!("test:{}", i + 1));
            assert_eq!(record.name, format!("test{}", i + 1));
            assert_eq!(record.value, (i + 1) as i32);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_schema_operations() -> Result<()> {
        let executor = setup_executor().await?;
        executor.enable_test_mode();

        // Create a schema change request
        let (tx, rx) = bounded(1);
        let schema_request = QueryRequest {
            query: "DEFINE TABLE test SCHEMAFULL".to_string(),
            params: vec![],
            priority: QueryPriority::Normal,
            query_type: QueryType::Schema,
            table_name: Some("test".to_string()),
            response_tx: tx,
        };

        // Execute schema change
        let result = executor.execute_schema_query(&schema_request).await?;
        assert!(result.is_object() || result.is_null());

        // Verify no other operations are running during schema change
        assert_eq!(
            executor.metrics.active_connections.load(Ordering::Relaxed),
            0
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_error_handling_and_retries() -> Result<()> {
        let executor = setup_executor().await?;
        executor.enable_test_mode();

        // Create a request that will fail
        let (tx, rx) = bounded(1);
        let bad_request = QueryRequest {
            query: "SELECT * FROM nonexistent_table".to_string(),
            params: vec![],
            priority: QueryPriority::Normal,
            query_type: QueryType::Read,
            table_name: None,
            response_tx: tx,
        };

        // Execute and verify it retries MAX_RETRIES times
        let result = executor.execute_with_retries(bad_request).await;
        assert!(result.is_err());

        // Check metrics
        assert_eq!(
            executor.metrics.queries_failed.load(Ordering::Relaxed),
            1
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_metrics_collection() -> Result<()> {
        let executor = setup_executor().await?;
        executor.enable_test_mode();

        // Execute some queries
        let (tx, _rx) = bounded(1);
        let request = QueryRequest {
            query: "CREATE test:1 SET name = 'test'".to_string(),
            params: vec![],
            priority: QueryPriority::Normal,
            query_type: QueryType::Write,
            table_name: Some("test".to_string()),
            response_tx: tx,
        };

        executor.execute_raw(request).await?;

        // Verify metrics
        let metrics = executor.metrics().await;
        assert!(metrics.queries_executed.load(Ordering::Relaxed) > 0);
        assert_eq!(metrics.queries_failed.load(Ordering::Relaxed), 0);
        assert!(metrics.total_query_time.load(Ordering::Relaxed) > 0);

        Ok(())
    }
}
