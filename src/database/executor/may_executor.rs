use std::sync::Arc;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use may::coroutine;
use may::sync::{mpsc::{channel, Receiver as MayReceiver, Sender as MaySender}};
use serde::de::DeserializeOwned;
use crate::database::executor::{Executor, ExecutorConfig, ExecutorMetrics, QueryRequest};
use crate::database::pool::connection::{SurrealConnection, SurrealConnectionManager};
use crate::database::runtime::{RuntimeManager, RuntimeType};
use crate::database::rw::RwLock;
use crate::database::scheduler::{QueryPriority, QueryType};
use crate::query::{Query, TransactionStatement};

pub struct MayExecutor {
    config: ExecutorConfig,
    manager: SurrealConnectionManager,
    metrics: Arc<RwLock<ExecutorMetrics>>,
    runtime: Arc<RuntimeManager>,
    shutdown_tx: MaySender<()>,
    shutdown_rx: MayReceiver<()>,
}

impl MayExecutor {
    pub fn new(config: ExecutorConfig, manager: SurrealConnectionManager, runtime: Arc<RuntimeManager>) -> Result<Self> {
        if runtime.may_config().is_none() {
            return Err(anyhow!("May runtime not initialized"));
        }
        
        let (shutdown_tx, shutdown_rx) = channel();
        Ok(Self {
            config: config.clone(),
            manager,
            metrics: Arc::new(RwLock::new(ExecutorMetrics::default())),
            runtime,
            shutdown_tx,
            shutdown_rx,
        })
    }
    
    async fn execute_transaction<T>(&self, queries: Vec<String>) -> Result<Vec<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        // Get connection from pool
        let conn = self.manager.get_conn().await?;
        
        // Update metrics
        {
            let mut metrics = self.metrics.write()?;
            metrics.active_connections += 1;
        }
        
        // Build transaction
        let mut tx = Query::begin();
        for query in queries {
            tx = tx.raw(&query);
        }
        tx = tx.commit();
        
        let sql = tx.build();
        
        // Execute transaction with timeout using channels
        let start = std::time::Instant::now();
        let mut result = {
            let (tx, rx) = channel();
            let query_future = conn.query(&sql);
            
            // Use scope to ensure coroutines are cleaned up
            coroutine::scope(|scope| unsafe {
                // Spawn timeout coroutine
                let timeout = self.config.query_timeout;
                scope.spawn(move || {
                    coroutine::sleep(timeout);
                    let _ = tx.send(Err(anyhow!("Transaction timeout")));
                    coroutine::yield_now();
                });
                
                // Spawn query execution coroutine
                scope.spawn(async move || {
                    match query_future.await {
                        Ok(result) => {
                            let _ = tx.send(Ok(result));
                        }
                        Err(e) => {
                            let _ = tx.send(Err(anyhow!(e)));
                        }
                    }
                    coroutine::yield_now();
                }.await);
            });
            
            // Wait for either query completion or timeout
            rx.recv()?
        }?;
        
        // Process all results from transaction
        let mut results = Vec::new();
        for i in 0..queries.len() {
            if let Ok(res) = result.take(i) {
                results.extend(res);
            }
        }
        
        // Update metrics
        {
            let mut metrics = self.metrics.write()?;
            metrics.active_connections -= 1;
            metrics.queries_executed += queries.len();
            metrics.avg_query_time = (metrics.avg_query_time * (metrics.queries_executed - queries.len()) as f64
                + start.elapsed().as_secs_f64())
                / metrics.queries_executed as f64;
        }
        
        Ok(results)
    }
    
    /// Batch multiple queries into a single transaction
    async fn batch_execute<T>(&self, requests: Vec<QueryRequest>) -> Result<Vec<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let queries: Vec<String> = requests.into_iter()
            .map(|r| r.query)
            .collect();
            
        self.execute_transaction(queries).await
    }
}

#[async_trait]
impl Executor for MayExecutor {
    async fn execute<T>(&self, request: QueryRequest) -> Result<Vec<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        // Always wrap single queries in transactions too
        self.execute_transaction(vec![request.query]).await
    }
    
    async fn execute_parallel<T>(&self, requests: Vec<QueryRequest>) -> Result<Vec<Result<Vec<T>>>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        // Group requests by priority
        use std::collections::HashMap;
        let mut priority_groups: HashMap<QueryPriority, Vec<QueryRequest>> = HashMap::new();
        
        for request in requests {
            priority_groups.entry(request.priority)
                .or_insert_with(Vec::new)
                .push(request);
        }
        
        let mut results = Vec::new();
        
        // Execute critical queries immediately in current coroutine
        if let Some(critical) = priority_groups.remove(&QueryPriority::Critical) {
            results.push(self.batch_execute(critical).await.map(|r| r));
        }
        
        // Execute other priority groups in parallel
        let (tx, rx) = channel();
        let executor = self.clone();
        
        coroutine::scope(|scope| {
            for (_, group) in priority_groups {
                let tx = tx.clone();
                let executor = executor.clone();

                unsafe {
                    scope.spawn(move || {
                        let result = executor.batch_execute(group);
                        let _ = tx.send(result);
                        coroutine::yield_now();
                    });
                }
            }
        });
        
        // Collect results
        while let Ok(result) = rx.recv() {
            results.push(result);
        }
        
        Ok(results)
    }
    
    async fn run(&self) -> Result<()> {
        // Create coroutine guard
        let _guard = self.runtime.create_guard()
            .ok_or_else(|| anyhow!("Failed to create coroutine guard"))?;
            
        // Main event loop
        loop {
            if self.shutdown_rx.try_recv().is_ok() {
                break;
            }
            
            // Yield to allow other coroutines to run
            coroutine::yield_now();
        }
        
        Ok(())
    }
    
    async fn stop(&self) -> Result<()> {
        self.shutdown_tx.send(())?;
        Ok(())
    }
    
    async fn metrics(&self) -> Result<ExecutorMetrics> {
        Ok(self.metrics.read()?.clone())
    }
}

impl Clone for MayExecutor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            manager: self.manager.clone(),
            metrics: self.metrics.clone(),
            runtime: self.runtime.clone(),
            shutdown_tx: self.shutdown_tx.clone(),
            shutdown_rx: self.shutdown_rx.iter().cloned().collect(),
        }
    }
}
