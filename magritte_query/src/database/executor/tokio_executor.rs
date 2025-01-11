use std::sync::Arc;
use std::time::Duration;
use anyhow::{anyhow, Result};
use async_channel::{bounded, Receiver, Sender};
use dashmap::DashMap;
use serde_json::Value;
use deadpool_surrealdb::{Object, Pool};
use surrealdb::sql::Value as DbValue;

#[cfg(feature = "rt-tokio")]
use tokio::sync::Notify;
#[cfg(feature = "rt-tokio")]
use tokio::time::sleep;
#[cfg(feature = "rt-tokio")]
use structured_spawn::spawn;
#[cfg(feature = "rt-tokio")]
use structured_spawn::TaskHandle;

use crate::database::executor::{BaseExecutor, ExecutorConfig, ExecutorMetrics, QueryRequest};
use crate::database::QueryType;
use crate::database::runtime::RuntimeManager;
use crate::database::rw::RwLock;
use crate::Query;

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 100;

#[derive(Debug)]
enum QueryError {
    ConnectionError(anyhow::Error),
    ExecutionError(anyhow::Error),
    TransactionError(anyhow::Error),
    RetryableError(anyhow::Error),
}

impl From<QueryError> for anyhow::Error {
    fn from(error: QueryError) -> Self {
        match error {
            QueryError::ConnectionError(e) => anyhow!("Connection error: {}", e),
            QueryError::ExecutionError(e) => anyhow!("Query execution error: {}", e),
            QueryError::TransactionError(e) => anyhow!("Transaction error: {}", e),
            QueryError::RetryableError(e) => anyhow!("Retryable error: {}", e),
        }
    }
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
    ) -> Result<Value> {
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
                Err(e) => return Err(e.into()),
            }
        }
        
        Err(anyhow!("Max retries exceeded: {}", last_error.unwrap()))
    }
    
    async fn execute_query(
        &self,
        request: QueryRequest,
    ) -> Result<Value, QueryError> {
        match request.query_type {
            QueryType::Read => self.execute_read(request).await,
            QueryType::Write => self.execute_write(request).await,
            QueryType::Schema => self.execute_schema(request).await,
        }
    }
    
    async fn execute_read(
        &self,
        request: QueryRequest,
    ) -> Result<Value, QueryError> {
        // Execute query with timeout
        let timeout = self.config.query_timeout;
        
        tokio::select! {
            result = async {
                let mut response = self.connection.query(&request.query)
                    .bind(request.params)
                    .await
                    .map_err(|e| QueryError::ExecutionError(anyhow!(e)))?;
                let db_value = response.take::<DbValue>(0)
                    .map_err(|e| QueryError::ExecutionError(anyhow!(e)))?;
                let value = serde_json::to_value(db_value)
                    .map_err(|e| QueryError::ExecutionError(anyhow!(e)))?;
                Ok::<Value, QueryError>(value)
            } => {
                result
            }
            _ = sleep(timeout) => {
                Err(QueryError::RetryableError(anyhow!("Query timeout")))
            }
        }
    }
    
    async fn execute_write(
        &self,
        request: QueryRequest,
    ) -> Result<Value, QueryError> {
        // Use Query::begin() for transaction
        let tx = Query::begin();
        let tx = tx.raw(&request.query);
        
        // Execute query within transaction with timeout
        let timeout = self.config.query_timeout;
        
        let result = tokio::select! {
            result = async {
                let mut response = tx.execute(self.connection.as_ref())
                    .await
                    .map_err(|e| QueryError::ExecutionError(anyhow!(e)))?;
                let db_value = response.take::<DbValue>(0)
                    .map_err(|e| QueryError::ExecutionError(anyhow!(e)))?;
                let value = serde_json::to_value(db_value)
                    .map_err(|e| QueryError::ExecutionError(anyhow!(e)))?;
                Ok::<Value, QueryError>(value)
            } => {
                result
            }
            _ = sleep(timeout) => {
                Err(QueryError::RetryableError(anyhow!("Query timeout")))
            }
        };
        
        result
    }
    
    async fn execute_schema(
        &self,
        request: QueryRequest,
    ) -> Result<Value, QueryError> {
        // Schema changes are executed atomically
        self.execute_write(request).await
    }
}

/// Tokio-based executor implementation
#[cfg(feature = "rt-tokio")]
pub struct TokioExecutor {
    config: ExecutorConfig,
    event_loops: DashMap<usize, (Sender<QueryRequest>, TaskHandle<()>)>,
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
            let handle = spawn(async move {
                let event_loop = EventLoop::new(i, rx, connection, metrics, notify, config);
                if let Err(e) = event_loop.run().await {
                    tracing::error!("Event loop {} failed: {}", i, e);
                }
            });
            
            self.event_loops.insert(i, (tx, handle));
            
            // Update metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.idle_connections += 1;
            }
        }
        
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        // Signal all event loops to shut down
        self.notify_shutdown.notify_waiters();
        
        // Wait for all event loops to finish
        for entry in self.event_loops.iter() {
            let (_, (_, handle)) = entry.pair();
            if let Err(e) = handle.await {
                tracing::error!("Failed to join event loop: {}", e);
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
    
    async fn execute_raw(&self, request: QueryRequest) -> Result<Value> {
        // Get next available event loop using round-robin
        let event_loop_count = self.event_loops.len();
        let event_loop_id = {
            let metrics = self.metrics.read().await;
            metrics.queries_executed % event_loop_count
        };
        
        // Get sender for chosen event loop
        let sender = self.event_loops.get(&event_loop_id)
            .map(|pair| pair.value().0.clone())
            .ok_or_else(|| anyhow!("Event loop not found"))?;
            
        // Send request to event loop
        sender.send(request)
            .await
            .map_err(|e| anyhow!("Failed to send request to event loop: {}", e))?;
            
        Ok(Value::Null) // Actual response will be sent through response_tx
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
            pool,
            runtime,
            metrics: Arc::new(RwLock::new(ExecutorMetrics::default())),
            notify_shutdown: Arc::new(Notify::new()),
        })
    }
}