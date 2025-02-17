use anyhow::Result;
use async_trait::async_trait;
use deadpool_surrealdb::Pool;
use futures_concurrency::prelude::*;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use structured_spawn::spawn;
use tokio::select;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, timeout};

use crate::database::executor::utils::query_batcher::{BatchResult, QueryBatch};
use crate::database::executor::{
    BaseExecutor, ExecutorConfig, ExecutorError, ExecutorMetrics, ExecutorState, QueryRequest,
};

#[derive(Debug)]
pub struct Executor {
    pool: Pool,
    config: ExecutorConfig,
    metrics: Arc<ExecutorMetrics>,
    state: Arc<RwLock<ExecutorState>>,
    query_tx: mpsc::Sender<QueryRequest>,
    suspend_tx: mpsc::Sender<bool>,
    batch: Arc<QueryBatch>,
}

#[derive(Debug)]
struct EventLoop {
    executor: Arc<Executor>,
    query_rx: mpsc::Receiver<QueryRequest>,
    suspend_rx: mpsc::Receiver<bool>,
}

impl EventLoop {
    fn new(
        executor: Arc<Executor>,
        query_rx: mpsc::Receiver<QueryRequest>,
        suspend_rx: mpsc::Receiver<bool>,
    ) -> Self {
        Self {
            executor,
            query_rx,
            suspend_rx,
        }
    }

    async fn run(mut self) -> Result<(), ExecutorError> {
        let mut interval = tokio::time::interval(self.executor.config.batch.batch_timeout);
        let mut suspended = false;

        loop {
            select! {
                _ = interval.tick() => {
                    if !suspended && self.executor.batch.should_flush().await {
                        let batch_result = self.executor.batch.flush().await;
                        if let Err(e) = self.executor.process_batch(batch_result).await {
                            tracing::error!("Failed to process batch: {}", e);
                        }
                    }
                }
                Some(resume) = self.suspend_rx.recv() => {
                    suspended = !resume;
                    if suspended {
                        // Don't try to flush on suspend - this could cause deadlocks
                        // Just mark as suspended and let the timeout handle any pending batches
                        tracing::info!("Executor suspended");
                    } else {
                        tracing::info!("Executor resumed");
                    }
                }
                Some(req) = self.query_rx.recv() => {
                    if suspended {
                        // When suspended, immediately return error without touching the batch
                        let error = anyhow::anyhow!("Executor is suspended");

                        // Use try_send instead of send to avoid blocking
                        if let Err(e) = req.response_tx.try_send(Err(error)) {
                            tracing::error!("Failed to send suspended error response: {}", e);
                        }
                        continue;
                    }

                    // Add timeout for batch operations to prevent hanging
                    match tokio::time::timeout(
                        Duration::from_millis(100),
                        self.executor.batch.add_request(req.clone())
                    ).await {
                        Ok(Ok(_)) => {
                            // Check if we should flush immediately
                            if self.executor.batch.should_flush().await {
                                let batch_result = self.executor.batch.flush().await;
                                if let Err(e) = self.executor.process_batch(batch_result).await {
                                    tracing::error!("Failed to process batch: {}", e);
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            // Batch is full or error occurred
                            tracing::error!("Failed to add request to batch: {}", e);
                            if let Err(e) = req.response_tx.send(Err(anyhow::anyhow!(e))).await {
                                tracing::error!("Failed to send error response: {}", e);
                            }
                        }
                        Err(_) => {
                            // Timeout occurred
                            if let Err(e) = req.response_tx.send(Err(anyhow::anyhow!("Batch operation timed out"))).await {
                                tracing::error!("Failed to send timeout error response: {}", e);
                            }
                        }
                    }
                }
                else => break,
            }
        }
        Ok(())
    }
}

impl Executor {
    pub async fn new(pool: Pool, config: ExecutorConfig) -> Result<Arc<Self>> {
        let (query_tx, query_rx) = mpsc::channel(config.query.max_concurrent_queries);
        let (suspend_tx, suspend_rx) = mpsc::channel(1);
        let (ready_tx, mut ready_rx) = mpsc::channel(1);
        let metrics = Arc::new(ExecutorMetrics::new());
        let batch = Arc::new(QueryBatch::new(config.batch.clone()));
        let executor = Self {
            pool,
            config,
            metrics,
            state: Arc::new(RwLock::new(ExecutorState::Starting)),
            query_tx,
            suspend_tx,
            batch,
        };

        let arc_executor = Arc::new(executor);
        // Start the event loop
        let event_loop = EventLoop::new(arc_executor.clone(), query_rx, suspend_rx);
        tokio::spawn(async move {
            // Signal that we're ready to process requests
            let _ = ready_tx.send(()).await;
            if let Err(e) = event_loop.run().await {
                tracing::error!("Event loop error: {}", e);
            }
        });

        // Wait for event loop to be ready
        ready_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to start event loop"))?;

        // Start the executor
        arc_executor.start().await?;

        Ok(arc_executor)
    }

    async fn process_batch(&self, batch_result: BatchResult) -> Result<(), ExecutorError> {
        // Process reads in parallel using futures-concurrency join
        if !batch_result.reads.is_empty() {
            let conn = self
                .pool
                .get()
                .await
                .map_err(|e| ExecutorError::ConnectionError(e.to_string()))?;

            let metrics = self.metrics.clone();
            let futures = batch_result
                .reads
                .into_iter()
                .map(|req| {
                    let conn = conn.clone();
                    let metrics = metrics.clone();
                    spawn(async move {
                        let start = std::time::Instant::now();
                        let result = {
                            let mut q = conn.query(&req.query);
                            if !req.params.is_empty() {
                                q = q.bind(req.params.into_iter().collect::<Vec<_>>())
                            }
                            q
                        }
                        .await
                        .and_then(|r| {
                            println!("Raw query response before check: {:?}", r);
                            r.check()
                        })
                        .and_then(|mut r| {
                            println!("Raw query response after check: {:?}", r);
                            // First try to extract as Option<surrealdb::Value> to handle None and complex objects
                            r.take::<Option<surrealdb::Value>>(0)
                        });

                        match result {
                            Ok(maybe_value) => {
                                metrics.update_success(start.elapsed().as_micros() as usize);
                                println!("Final response from SurrealDB: {:?}", maybe_value);

                                // Handle both None and Some cases
                                let json_values = match maybe_value {
                                    Some(value) => {
                                        // Convert to serde_json::Value first
                                        let json_value = serde_json::to_value(&value).unwrap_or(serde_json::Value::Null);

                                        // If it's already an array, use it directly; otherwise, wrap in an array
                                        if let serde_json::Value::Array(arr) = json_value {
                                            arr
                                        } else {
                                            vec![json_value]
                                        }
                                    },
                                    None => vec![], // Empty array for None
                                };

                                // Always return as array
                                let response_value = serde_json::Value::Array(json_values);

                                if let Err(e) = req.response_tx.send(Ok(response_value)).await {
                                    tracing::error!("Failed to send query response: {}", e);
                                }
                            }
                            Err(e) => {
                                metrics.update_failure();
                                println!("Error from SurrealDB: {:?}", e);
                                if let Err(e) = req.response_tx.send(Err(anyhow::anyhow!(e))).await {
                                    tracing::error!("Failed to send query error: {}", e);
                                }
                            }
                        }
                    })
                })
                .collect::<Vec<_>>();

            futures.join().await;
        }

        // Process writes sequentially per table using structured_spawn
        for (table, writes) in batch_result.writes.into_iter() {
            let conn = self
                .pool
                .get()
                .await
                .map_err(|e| ExecutorError::ConnectionError(e.to_string()))?;

            let metrics = self.metrics.clone();
            for req in writes {
                let conn = conn.clone();
                let metrics = metrics.clone();
                spawn(async move {
                    let start = std::time::Instant::now();
                    let result = {
                        let mut q = conn.query(&req.query);
                        if !req.params.is_empty() {
                            q = q.bind(req.params.into_iter().collect::<Vec<_>>())
                        }
                        q
                    }
                    .await
                    .and_then(|r| {
                        println!("Raw query response before check: {:?}", r);
                        r.check()
                    })
                    .and_then(|mut r| {
                        println!("Raw query response after check: {:?}", r);
                        // First try to extract as Option<surrealdb::Value> to handle None and complex objects
                        r.take::<Option<surrealdb::Value>>(0)
                    });

                    match result {
                        Ok(maybe_value) => {
                            metrics.update_success(start.elapsed().as_micros() as usize);
                            println!("Final response from SurrealDB: {:?}", maybe_value);

                            // Handle both None and Some cases
                            let json_values = match maybe_value {
                                Some(value) => {
                                    // Convert to serde_json::Value first
                                    let json_value = serde_json::to_value(&value).unwrap_or(serde_json::Value::Null);

                                    // If it's already an array, use it directly; otherwise, wrap in an array
                                    if let serde_json::Value::Array(arr) = json_value {
                                        arr
                                    } else {
                                        vec![json_value]
                                    }
                                },
                                None => vec![], // Empty array for None
                            };

                            // Always return as array
                            let response_value = serde_json::Value::Array(json_values);
                            
                            if let Err(e) = req.response_tx.send(Ok(response_value)).await {
                                tracing::error!("Failed to send query response: {}", e);
                            }
                        }
                        Err(e) => {
                            metrics.update_failure();
                            println!("Error from SurrealDB: {:?}", e);
                            if let Err(e) = req.response_tx.send(Err(anyhow::anyhow!(e))).await {
                                tracing::error!("Failed to send query error: {}", e);
                            }
                        }
                    }
                })
                .await;
            }
        }

        // Process schema changes serially
        if !batch_result.schema.is_empty() {
            let conn = self
                .pool
                .get()
                .await
                .map_err(|e| ExecutorError::ConnectionError(e.to_string()))?;

            let metrics = self.metrics.clone();
            for req in batch_result.schema {
                let conn = conn.clone();
                let metrics = metrics.clone();
                let start = std::time::Instant::now();
                let result = {
                    let mut q = conn.query(&req.query);
                    if !req.params.is_empty() {
                        q = q.bind(req.params.into_iter().collect::<Vec<_>>())
                    }
                    q
                }
                .await
                .and_then(|r| {
                    println!("Raw query response before check: {:?}", r);
                    r.check()
                });

                // For schema queries, don't try to extract the response value
                // Just check if the query succeeded (result is Ok)
                match result {
                    Ok(_) => {
                        metrics.update_success(start.elapsed().as_micros() as usize);
                        
                        // Return an empty array for schema queries since we just care about success/failure
                        let response_value = serde_json::Value::Array(vec![]);
                        
                        if let Err(e) = req.response_tx.send(Ok(response_value)).await {
                            tracing::error!("Failed to send query response: {}", e);
                        }
                    }
                    Err(e) => {
                        metrics.update_failure();
                        println!("Error from SurrealDB: {:?}", e);
                        if let Err(e) = req.response_tx.send(Err(anyhow::anyhow!(e))).await {
                            tracing::error!("Failed to send query error: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn suspend(&self) -> Result<(), ExecutorError> {
        self.suspend_tx
            .send(false)
            .await
            .map_err(|_| ExecutorError::ChannelError("Failed to suspend executor".into()))
    }

    pub async fn resume(&self) -> Result<(), ExecutorError> {
        self.suspend_tx
            .send(true)
            .await
            .map_err(|_| ExecutorError::ChannelError("Failed to resume executor".into()))
    }
}

#[async_trait]
impl BaseExecutor for Executor {
    async fn start(&self) -> Result<(), ExecutorError> {
        *self.state.write().await = ExecutorState::Running;
        self.resume().await?;
        Ok(())
    }

    async fn stop(&self) -> Result<(), ExecutorError> {
        *self.state.write().await = ExecutorState::ShuttingDown;

        // Suspend processing first
        self.suspend().await?;

        // Wait for pending queries with timeout
        if let Err(_) = timeout(Duration::from_secs(5), self.query_tx.closed()).await {
            tracing::warn!("Executor shutdown timed out, some queries may be lost");
        }

        *self.state.write().await = ExecutorState::Stopped;
        Ok(())
    }

    async fn metrics(&self) -> Arc<ExecutorMetrics> {
        self.metrics.clone()
    }

    async fn execute(&self, request: QueryRequest) -> Result<Value, ExecutorError> {
        if *self.state.read().await != ExecutorState::Running {
            return Err(ExecutorError::ExecutionError("Executor not running".into()));
        }

        self.query_tx
            .send(request)
            .await
            .map_err(|_| ExecutorError::ChannelError("Failed to send query".into()))?;

        Ok(Value::Null) // Actual response will be sent through the response channel
    }

    async fn is_healthy(&self) -> bool {
        matches!(*self.state.read().await, ExecutorState::Running)
    }

    async fn state(&self) -> ExecutorState {
        self.state.read().await.clone()
    }
}

impl Clone for Executor {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            config: self.config.clone(),
            metrics: self.metrics.clone(),
            state: self.state.clone(),
            query_tx: self.query_tx.clone(),
            suspend_tx: self.suspend_tx.clone(),
            batch: self.batch.clone(),
        }
    }
}
