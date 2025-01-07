#[cfg(feature = "may")]
mod may_executor;
#[cfg(all(feature = "may", feature = "coroutines"))]
mod may_coroutine_executor;

#[cfg(all(feature = "tokio", feature = "coroutines"))]
mod tokio_coroutine_executor;
#[cfg(feature = "glommio")]
mod glommio_executor;

#[cfg(all(feature = "glommio", feature = "coroutines"))]
mod glommio_coroutine_executor;
#[cfg(feature = "coroutines")]
mod coroutine_executor;
#[cfg(not(any(feature = "may", feature = "tokio", feature = "glommio", feature = "coroutines")))]
mod thread_executor;
pub(crate) mod tokio_executor;
mod async_std_executor;

use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use async_channel::{Sender, Receiver};
use serde::de::DeserializeOwned;
use serde_json::Value;
use crate::query::Query;
use crate::database::pool::connection::SurrealConnection;
use crate::database::scheduler::{QueryPriority, QueryType, ScheduledQuery};

/// Configuration for query execution
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Connection timeout
    pub connection_timeout: std::time::Duration,
    /// Query timeout
    pub query_timeout: std::time::Duration,
    /// Whether to use prepared statements
    pub use_prepared_statements: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_connections: 32,
            connection_timeout: std::time::Duration::from_secs(30),
            query_timeout: std::time::Duration::from_secs(30),
            use_prepared_statements: true,
        }
    }
}

/// A request to execute a query with its response channel
#[derive(Debug)]
pub struct QueryRequest {
    pub query: String,
    pub params: Vec<(String, Value)>,
    pub priority: QueryPriority,
    pub query_type: QueryType,
    pub response_tx: Sender<Result<Value>>,
}

impl From<ScheduledQuery> for QueryRequest {
    fn from(scheduled: ScheduledQuery) -> Self {
        Self {
            query: scheduled.query,
            params: scheduled.params,
            priority: scheduled.priority,
            query_type: scheduled.query_type,
            response_tx: scheduled.response_tx,
        }
    }
}

/// Executor metrics for monitoring
#[derive(Debug, Clone, Default)]
pub struct ExecutorMetrics {
    pub active_connections: usize,
    pub idle_connections: usize,
    pub queries_executed: usize,
    pub queries_failed: usize,
    pub avg_query_time: f64,
}

/// The core executor trait that all runtime-specific executors must implement
#[async_trait]
pub trait Executor: Send + Sync + 'static {
    /// Execute a query and deserialize the result into a vector of type T
    async fn execute<T>(&self, request: QueryRequest) -> Result<Vec<T>> 
    where 
        T: DeserializeOwned + Send + 'static;

    /// Start the executor's event loop
    async fn run(&self) -> Result<()>;
    
    /// Stop the executor
    async fn stop(&self) -> Result<()>;
    
    /// Get executor metrics
    async fn metrics(&self) -> ExecutorMetrics;
    
    /// Execute multiple queries in parallel
    async fn execute_parallel<T>(&self, requests: Vec<QueryRequest>) -> Result<Vec<Result<Vec<T>>>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let mut results = Vec::with_capacity(requests.len());
        let mut handles = Vec::with_capacity(requests.len());
        
        for request in requests {
            let executor = self.clone();
            let handle = tokio::spawn(async move {
                executor.execute::<T>(request).await
            });
            handles.push(handle);
        }
        
        for handle in handles {
            results.push(handle.await?);
        }
        
        Ok(results)
    }
    
    /// Execute a batch of write queries atomically
    async fn execute_atomic<T>(&self, requests: Vec<QueryRequest>) -> Result<Vec<Vec<T>>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        // Start transaction
        let mut results = Vec::with_capacity(requests.len());
        
        // Execute each query in sequence
        for request in requests {
            let result = self.execute::<T>(request).await?;
            results.push(result);
        }
        
        Ok(results)
    }
}