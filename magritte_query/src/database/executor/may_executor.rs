use std::sync::Arc;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use may::coroutine::{self, scope};
use may::sync::mpsc::{channel, Sender as MaySender};
use serde::de::DeserializeOwned;
use crate::database::executor::{BaseExecutor, ExecutorConfig, ExecutorMetrics, QueryRequest};
use crate::database::runtime::RuntimeManager;
use crate::database::QueryType;
use crate::query::Query;
use std::time::{Duration, Instant};
use deadpool_surrealdb::Pool;
use thiserror::Error;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use async_channel;

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 100;
const MAX_CONCURRENT: usize = 32;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Query execution failed: {0}")]
    ExecutionError(String),
    #[error("Query timed out")]
    Timeout,
    #[error("Connection error: {0}")]
    ConnectionError(String),
}

#[derive(Default)]
struct Metrics {
    active_connections: AtomicUsize,
    idle_connections: AtomicUsize,
    queries_executed: AtomicUsize,
    queries_failed: AtomicUsize,
    total_query_time: AtomicUsize,
}

pub struct MayExecutor {
    config: ExecutorConfig,
    pool: Pool,
    runtime: Arc<RuntimeManager>,
    metrics: Arc<Metrics>,
    running: Arc<AtomicBool>,
    request_tx: MaySender<QueryRequest>,
    active_tasks: Arc<AtomicUsize>,
}

impl MayExecutor {
    pub fn new(config: ExecutorConfig, pool: Pool, runtime: Arc<RuntimeManager>) -> Result<Self> {
        let (request_tx, request_rx) = channel();
        
        let executor = Self {
            config,
            pool,
            runtime,
            metrics: Arc::new(Metrics::default()),
            running: Arc::new(AtomicBool::new(false)),
            request_tx,
            active_tasks: Arc::new(AtomicUsize::new(0)),
        };

        // Start the event loop coroutine
        let executor_clone = executor.clone();
        unsafe {
            coroutine::spawn(move || {
                scope(|scope| {
                    scope.spawn(async move || {
                        if let Err(e) = executor_clone.run_event_loop(request_rx).await {
                            tracing::error!("Event loop error: {}", e);
                        }
                    });
                });
            });
        }

        Ok(executor)
    }

    async fn run_event_loop(&self, request_rx: may::sync::mpsc::Receiver<QueryRequest>) -> Result<()> {
        self.running.store(true, Ordering::SeqCst);

        while self.running.load(Ordering::SeqCst) {
            if self.active_tasks.load(Ordering::SeqCst) >= MAX_CONCURRENT {
                coroutine::sleep(Duration::from_millis(10));
                continue;
            }

            match request_rx.try_recv() {
                Ok(request) => {
                    self.active_tasks.fetch_add(1, Ordering::SeqCst);
                    let executor = self.clone();
                    let start_time = Instant::now();

                    unsafe {
                        scope(|scope| {
                            scope.spawn(async move || {
                                let result = match request.query_type {
                                    QueryType::Read => {
                                        executor.execute_read_query(&request).await
                                    }
                                    QueryType::Write => {
                                        executor.execute_write_query(&request).await
                                    }
                                    QueryType::Schema => {
                                        executor.execute_schema_query(&request).await
                                    }
                                };

                                executor.handle_query_result(result, start_time, &request).await;
                                executor.active_tasks.fetch_sub(1, Ordering::SeqCst);
                            });
                        });
                    }
                }
                Err(_) => {
                    coroutine::sleep(Duration::from_millis(10));
                }
            }
        }

        // Wait for active tasks to complete
        let mut prev_active_tasks = self.active_tasks.load(Ordering::SeqCst);
        while self.active_tasks.load(Ordering::SeqCst) > 0 {
            coroutine::sleep(Duration::from_millis(10));
            let current_tasks = self.active_tasks.load(Ordering::SeqCst);
            if current_tasks == prev_active_tasks {
                tracing::warn!("Waiting for {} active tasks to complete", current_tasks);
            }
            prev_active_tasks = current_tasks;
        }

        Ok(())
    }

    async fn handle_query_result(
        &self,
        result: Result<serde_json::Value, QueryError>,
        start_time: Instant,
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

    async fn execute_read_query(&self, request: &QueryRequest) -> Result<serde_json::Value, QueryError> {
        self.metrics.active_connections.fetch_add(1, Ordering::Relaxed);
        
        let conn = self.pool.get().await
            .map_err(|e| QueryError::ConnectionError(e.to_string()))?;

        let mut response = conn.query(request.query.as_str())
            .bind(request.params.clone())
            .await
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        self.metrics.active_connections.fetch_sub(1, Ordering::Relaxed);

        let value = response.take::<Option<surrealdb::Value>>(0)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?
            .ok_or_else(|| QueryError::ExecutionError("No value returned".to_string()))?;

        serde_json::to_value(value).map_err(|e| QueryError::ExecutionError(e.to_string()))
    }

    async fn execute_write_query(&self, request: &QueryRequest) -> Result<serde_json::Value, QueryError> {
        self.metrics.active_connections.fetch_add(1, Ordering::Relaxed);

        let conn = self.pool.get().await
            .map_err(|e| QueryError::ConnectionError(e.to_string()))?;

        let transaction = if let Some(table) = &request.table_name {
            let mut stmt = Query::begin();
            stmt = stmt.raw(&request.query);
            stmt.commit().build()
        } else {
            request.query.clone()
        };

        let mut response = conn.query(transaction)
            .bind(request.params.clone())
            .await
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        self.metrics.active_connections.fetch_sub(1, Ordering::Relaxed);

        let value = response.take::<Option<surrealdb::sql::Value>>(0)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?
            .ok_or_else(|| QueryError::ExecutionError("No value returned".to_string()))?;

        serde_json::to_value(value).map_err(|e| QueryError::ExecutionError(e.to_string()))
    }

    async fn execute_schema_query(&self, request: &QueryRequest) -> Result<serde_json::Value, QueryError> {
        // Wait for all active operations to complete
        while self.metrics.active_connections.load(Ordering::Relaxed) > 0 {
            coroutine::sleep(Duration::from_millis(50));
        }

        self.execute_write_query(request).await
    }

    async fn execute_raw(&self, request: QueryRequest) -> Result<serde_json::Value> {
        let (response_tx, response_rx) = async_channel::bounded(1);
        let request = QueryRequest {
            response_tx,
            ..request
        };

        self.request_tx.send(request)?;

        // Wait for response using may's coroutine
        unsafe {
            let (tx, rx) = may::sync::mpsc::channel();
            
            scope(|scope| {
                let tx = tx.clone();
                scope.spawn(async move || {
                    if let Ok(response) = response_rx.recv().await {
                        let _ = tx.send(response);
                    }
                });
            });
            
            match rx.recv() {
                Ok(Ok(value)) => Ok(value),
                Ok(Err(e)) => Err(e),
                Err(_) => Err(anyhow!("No response received")),
            }
        }
    }
}

#[async_trait]
impl BaseExecutor for MayExecutor {
    async fn run(&self) -> Result<()> {
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        
        while self.metrics.active_connections.load(Ordering::Relaxed) > 0 {
            coroutine::sleep(Duration::from_millis(100));
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

    async fn execute_raw(&self, request: QueryRequest) -> Result<serde_json::Value> {
        let (response_tx, response_rx) = async_channel::bounded(1);
        let request = QueryRequest {
            response_tx,
            ..request
        };

        self.request_tx.send(request)?;

        // Wait for response using may's coroutine
        unsafe {
            let (tx, rx) = may::sync::mpsc::channel();
            
            scope(|scope| {
                let tx = tx.clone();
                scope.spawn(async move || {
                    if let Ok(response) = response_rx.recv().await {
                        let _ = tx.send(response);
                    }
                });
            });
            
            match rx.recv() {
                Ok(Ok(value)) => Ok(value),
                Ok(Err(e)) => Err(e),
                Err(_) => Err(anyhow!("No response received")),
            }
        }
    }
}

impl Clone for MayExecutor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pool: self.pool.clone(),
            runtime: self.runtime.clone(),
            metrics: self.metrics.clone(),
            running: self.running.clone(),
            request_tx: self.request_tx.clone(),
            active_tasks: self.active_tasks.clone(),
        }
    }
}
