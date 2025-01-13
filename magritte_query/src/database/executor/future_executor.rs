use crate::database::executor::{BaseExecutor, ExecutorConfig, ExecutorMetrics, QueryRequest};
use crate::database::runtime::RuntimeManager;
use crate::database::QueryType;
use anyhow::{anyhow, Result};
use async_channel::{bounded, Receiver, Sender};
use deadpool_surrealdb::{Pool};
use futures_timer::Delay;
use futures_util::StreamExt;
use futures_util::future::{FutureExt, select, try_join_all};
use futures_locks::RwLock;
use serde_json::{Value as JsonValue, Value};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashSet;
use dashmap::DashMap;
use thiserror::Error;
use crate::Query;

const CHANNEL_SIZE: usize = 1000;
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 100;

#[cfg(test)]
const TEST_MAX_ITERATIONS: usize = 100;

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
    pub fn new(
        config: ExecutorConfig,
        pool: Pool,
        runtime: Arc<RuntimeManager>,
    ) -> Result<Self> {
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
        
        let mut tasks = Vec::new();

        while self.running.load(Ordering::SeqCst) {
            #[cfg(test)]
            if self.test_mode {
                let count = self.iteration_count.fetch_add(1, Ordering::SeqCst);
                if count >= TEST_MAX_ITERATIONS {
                    break;
                }
            }

            let timeout = Box::pin(Delay::new(Duration::from_millis(100)));
            let request_future = Box::pin(request_rx.recv().fuse());

            match select(request_future, timeout).await {
                futures_util::future::Either::Left((Ok(request), _)) => {
                    let executor = self;
                    let task = async move {
                        let start_time = std::time::Instant::now();
                        
                        let result = executor.execute_with_retries(request.clone()).await;

                        match result {
                            Ok(value) => {
                                executor.handle_query_result(Ok(value), start_time, &request).await;
                            }
                            Err(e) => {
                                executor.handle_query_result(Err(e), start_time, &request).await;
                            }
                        }
                        Ok::<_, anyhow::Error>(())
                    };
                    tasks.push(task);

                    if tasks.len() >= self.config.max_batch_size {
                        try_join_all(tasks.split_off(0)).await?;
                    }
                }
                futures_util::future::Either::Left((Err(_), _)) => break, // Channel closed
                futures_util::future::Either::Right(_) => {
                    if !tasks.is_empty() {
                        try_join_all(tasks.split_off(0)).await?;
                    }
                }
            }
        }

        if !tasks.is_empty() {
            try_join_all(tasks).await?;
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
                self.metrics.queries_executed.fetch_add(1, Ordering::Relaxed);
                let elapsed_micros = elapsed.as_micros() as usize;
                self.metrics.total_query_time.fetch_add(elapsed_micros, Ordering::Relaxed);

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
        let read_futures: Vec<_> = requests
            .into_iter()
            .map(|request| {
                let request = request.clone();
                Box::pin(async move { self.execute_read_query(&request).await })
            })
            .collect();

        try_join_all(read_futures).await
    }

    async fn execute_batched_writes(&self, requests: Vec<QueryRequest>, table_name: String) -> Result<Vec<JsonValue>, QueryError> {
        self.metrics.active_connections.fetch_add(1, Ordering::Relaxed);
        self.metrics.idle_connections.fetch_sub(1, Ordering::Relaxed);

        let connection = self.pool.get().await
            .map_err(|e| QueryError::ConnectionError(e.to_string()))?;

        let mut transaction = Query::begin();
        
        for request in &requests {
            transaction = transaction.raw(&request.query);
        }
        
        let transaction = transaction.commit().build();

        let mut response = connection
            .query(transaction)
            .bind(requests.iter().flat_map(|r| r.params.clone()).collect::<Vec<_>>())
            .await
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        self.metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
        self.metrics.idle_connections.fetch_add(1, Ordering::Relaxed);

        let mut results = Vec::with_capacity(requests.len());
        for i in 0..requests.len() {
            let value = response.take::<Option<surrealdb::sql::Value>>(i)
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
            },
            QueryType::Write => {
                if let Some(table_name) = &request.table_name {
                    if let Some(batch) = self.try_batch_writes(request, table_name).await {
                        let results = self.execute_batched_writes(batch, table_name.clone()).await?;
                        Ok(results[0].clone())
                    } else {
                        self.execute_write_query(request).await
                    }
                } else {
                    self.execute_write_query(request).await
                }
            },
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

    async fn try_batch_writes(&self, request: &QueryRequest, table_name: &str) -> Option<Vec<QueryRequest>> {
        let mut batch = vec![request.clone()];
        let batch_timeout = Duration::from_millis(self.config.batch_timeout_ms);
        let batch_start = std::time::Instant::now();
        
        while batch.len() < self.config.max_batch_size && batch_start.elapsed() < batch_timeout {
            match self.request_rx.try_recv() {
                Ok(req) => {
                    if req.query_type == QueryType::Write && 
                       req.table_name.as_deref() == Some(table_name) {
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
        self.metrics.active_connections.fetch_add(1, Ordering::Relaxed);
        self.metrics.idle_connections.fetch_sub(1, Ordering::Relaxed);

        let connection = self.pool.get().await
            .map_err(|e| QueryError::ConnectionError(e.to_string()))?;
            
        let mut response = connection
            .query(request.query.as_str())
            .bind(request.params.clone())
            .await
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        self.metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
        self.metrics.idle_connections.fetch_add(1, Ordering::Relaxed);

        let value = response.take::<Option<surrealdb::Value>>(0)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?
            .ok_or_else(|| QueryError::ExecutionError("No value returned".to_string()))?;
            
        serde_json::to_value(value)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))
    }

    async fn execute_write_query(&self, request: &QueryRequest) -> Result<JsonValue, QueryError> {
        self.metrics.active_connections.fetch_add(1, Ordering::Relaxed);
        self.metrics.idle_connections.fetch_sub(1, Ordering::Relaxed);

        let connection = self.pool.get().await
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

        self.metrics.active_connections.fetch_sub(1, Ordering::Relaxed);
        self.metrics.idle_connections.fetch_add(1, Ordering::Relaxed);

        let value = response.take::<Option<surrealdb::sql::Value>>(0)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?
            .ok_or_else(|| QueryError::ExecutionError("No value returned".to_string()))?;
            
        serde_json::to_value(value)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))
    }

    async fn execute_schema_query(&self, request: &QueryRequest) -> Result<JsonValue, QueryError> {
        // Schema operations should also be atomic
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
            active_connections: AtomicUsize::new(self.metrics.active_connections.load(Ordering::Relaxed)),
            idle_connections: AtomicUsize::new(self.metrics.idle_connections.load(Ordering::Relaxed)),
            queries_executed: AtomicUsize::new(self.metrics.queries_executed.load(Ordering::Relaxed)),
            queries_failed: AtomicUsize::new(self.metrics.queries_failed.load(Ordering::Relaxed)),
            total_query_time: AtomicUsize::new(self.metrics.total_query_time.load(Ordering::Relaxed)),
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
