use crate::database::executor::{BaseExecutor, ExecutorConfig, ExecutorMetrics, QueryRequest};
use crate::database::runtime::RuntimeManager;
use crate::database::QueryType;
use crate::Query;
use anyhow::{anyhow, Result};
use async_channel::{bounded, Receiver, Sender};
use dashmap::DashMap;
use deadpool_surrealdb::{Object, Pool};
use futures_concurrency::prelude::*;
use futures_executor::block_on;
use futures_timer::Delay;
use futures_util::{Stream, StreamExt};
use serde_json::Value as JsonValue;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use surrealdb::sql::Value;
use thiserror::Error;

const CHANNEL_SIZE: usize = 1000;
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 100;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Query execution failed: {0}")]
    ExecutionError(String),
    #[error("Query timed out")]
    Timeout,
    #[error("Connection error: {0}")]
    ConnectionError(String),
}

struct EventLoop {
    id: usize,
    rx: Receiver<QueryRequest>,
    connection: Object,
    metrics: Arc<Mutex<ExecutorMetrics>>,
    config: ExecutorConfig,
}

impl EventLoop {
    fn new(
        id: usize,
        rx: Receiver<QueryRequest>,
        connection: Object,
        metrics: Arc<Mutex<ExecutorMetrics>>,
        config: ExecutorConfig,
    ) -> Self {
        Self {
            id,
            rx,
            connection,
            metrics,
            config,
        }
    }

    fn run(&self) -> Result<()> {
        block_on(async {
            let mut consecutive_errors = 0;

            while let Ok(request) = self.rx.recv().await {
                let start = std::time::Instant::now();
                let result = self.execute_with_retries(request.clone()).await;
                
                // Update metrics
                if let Ok(mut metrics) = self.metrics.lock() {
                    metrics.queries_executed += 1;
                    metrics.avg_query_time = (metrics.avg_query_time 
                        * (metrics.queries_executed - 1) as f64
                        + start.elapsed().as_secs_f64()) 
                        / metrics.queries_executed as f64;
                    
                    match &result {
                        Ok(_) => consecutive_errors = 0,
                        Err(e) => {
                            metrics.queries_failed += 1;
                            consecutive_errors += 1;
                            tracing::error!("Query execution failed on event loop {}: {}", self.id, e);
                        }
                    }
                }
                
                let result = result.map_err(|e| anyhow!(e));
                if let Err(e) = request.response_tx.send(result).await {
                    tracing::error!("Failed to send query response: {}", e);
                }
            }
            
            Ok(())
        })
    }

    async fn execute_with_retries(&self, request: QueryRequest) -> Result<JsonValue, QueryError> {
        let mut attempts = 0;
        loop {
            match self.execute_query(&request).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    if attempts >= MAX_RETRIES {
                        return Err(e);
                    }
                    Delay::new(Duration::from_millis(RETRY_DELAY_MS * 2u64.pow(attempts))).await;
                }
            }
        }
    }

    async fn execute_query(&self, request: &QueryRequest) -> Result<JsonValue, QueryError> {
        match request.query_type {
            QueryType::Read => self.execute_read(request).await,
            QueryType::Write => self.execute_write(request).await,
            QueryType::Schema => self.execute_schema(request).await,
        }
    }

    async fn execute_read(&self, request: &QueryRequest) -> Result<JsonValue, QueryError> {
        let mut response = self.connection.query(request.query.as_str())
            .bind(request.params.clone())
            .await
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        let db_value = response.take::<Option<Value>>(0)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?
            .unwrap_or_default();

        serde_json::to_value(db_value)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))
    }

    async fn execute_write(&self, request: &QueryRequest) -> Result<JsonValue, QueryError> {
        let mut response = self.connection.query(request.query.as_str())
            .bind(request.params.clone())
            .await
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        let db_value = response.take::<Option<Value>>(0)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?
            .unwrap_or_default();

        serde_json::to_value(db_value)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))
    }

    async fn execute_schema(&self, request: &QueryRequest) -> Result<JsonValue, QueryError> {
        let mut response = self.connection.query(request.query.as_str())
            .bind(request.params.clone())
            .await
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        let db_value = response.take::<Option<Value>>(0)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?
            .unwrap_or_default();

        serde_json::to_value(db_value)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))
    }
}

pub struct FutureExecutor {
    config: ExecutorConfig,
    event_loops: DashMap<usize, Sender<QueryRequest>>,
    pool: Pool,
    runtime: Arc<RuntimeManager>,
    metrics: Arc<Mutex<ExecutorMetrics>>,
}

#[async_trait::async_trait]
impl BaseExecutor for FutureExecutor {
    async fn run(&self) -> Result<()> {
        let futures = (0..self.config.max_connections)
            .into_iter()
            .map(|i| {
                let metrics = self.metrics.clone();
                let pool = self.pool.clone();
                let config = self.config.clone();
                let event_loops = self.event_loops.clone();
                
                async move {
                    let (tx, rx) = bounded(CHANNEL_SIZE);
                    let connection = pool.get().await?;
                    
                    let event_loop = EventLoop::new(i, rx, connection, metrics.clone(), config);
                    event_loops.insert(i, tx);
                    
                    if let Ok(mut metrics) = metrics.lock() {
                        metrics.idle_connections += 1;
                    }
                    
                    event_loop.run()
                }
            });
        
        futures.collect::<Vec<_>>().join().await;
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        self.event_loops.clear();
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.active_connections = 0;
            metrics.idle_connections = 0;
        }
        Ok(())
    }

    async fn metrics(&self) -> Arc<ExecutorMetrics> {
        if let Ok(metrics) = self.metrics.lock() {
            Arc::new((*metrics).clone())
        } else {
            Arc::new(ExecutorMetrics::default())
        }
    }

    async fn execute_raw(&self, request: QueryRequest) -> Result<JsonValue> {
        let event_loop_count = self.event_loops.len();
        let event_loop_id = if let Ok(metrics) = self.metrics.lock() {
            metrics.queries_executed % event_loop_count
        } else {
            0
        };
        
        let sender = self.event_loops
            .get(&event_loop_id)
            .map(|pair| pair.value().clone())
            .ok_or_else(|| anyhow!("Event loop not found"))?;
            
        sender.send(request)
            .await
            .map_err(|e| anyhow!("Failed to send request to event loop: {}", e))?;
            
        Ok(JsonValue::Null)
    }
}

impl FutureExecutor {
    pub fn new(config: ExecutorConfig, pool: Pool, runtime: Arc<RuntimeManager>) -> Result<Self> {
        Ok(Self {
            config,
            event_loops: DashMap::new(),
            pool,
            runtime,
            metrics: Arc::new(Mutex::new(ExecutorMetrics::default())),
        })
    }
}
