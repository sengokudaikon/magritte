pub(crate) mod future_executor;
pub(crate) mod crossbeam_executor;
pub(crate) mod coroutine_executor;
pub(crate) mod rayon_executor;

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use anyhow::Result;
use async_channel::Sender;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::Value;
use crate::database::QueryType;
use crate::database::scheduler::{QueryPriority, ScheduledQuery};

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
    /// Maximum batch size for read/write operations
    pub max_batch_size: usize,
    /// Timeout for batching operations in milliseconds
    pub batch_timeout_ms: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_connections: 32,
            connection_timeout: std::time::Duration::from_secs(30),
            query_timeout: std::time::Duration::from_secs(30),
            use_prepared_statements: true,
            max_batch_size: 100,
            batch_timeout_ms: 50,
        }
    }
}

/// A request to execute a query with its response channel
#[derive(Debug, Clone)]
pub struct QueryRequest {
    pub query: String,
    pub params: Vec<(String, Value)>,
    pub priority: QueryPriority,
    pub query_type: QueryType,
    pub table_name: Option<String>,
    pub response_tx: Sender<Result<Value>>,
}

impl From<ScheduledQuery> for QueryRequest {
    fn from(query: ScheduledQuery) -> Self {
        Self {
            query: query.query,
            params: query.params,
            priority: query.priority,
            query_type: query.query_type,
            table_name: query.table_name,
            response_tx: query.response_tx,
        }
    }
}

/// Base executor trait for runtime-agnostic operations
#[async_trait]
pub trait BaseExecutor: Send {
    /// Start the executor
    async fn run(&self) -> Result<()>;
    
    /// Stop the executor
    async fn stop(&self) -> Result<()>;
    
    /// Get executor metrics
    async fn metrics(&self) -> Arc<ExecutorMetrics>;
    
    /// Execute a raw query and return deserialized results.
    /// The executor will:
    /// 1. Detect query type (read/write)
    /// 2. Choose execution strategy (parallel for reads, atomic for writes)
    /// 3. Handle connection management
    /// 4. Apply query prioritization
    async fn execute_raw(&self, request: QueryRequest) -> Result<Value>;
}

/// Executor metrics for monitoring
#[derive(Debug, Default)]
pub struct ExecutorMetrics {
    pub active_connections: AtomicUsize,
    pub idle_connections: AtomicUsize,
    pub queries_executed: AtomicUsize,
    pub queries_failed: AtomicUsize,
    pub total_query_time: AtomicUsize,
}
