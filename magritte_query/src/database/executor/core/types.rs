use anyhow::Result;
use serde_json::Value;
use thiserror::Error;
use tokio::sync::mpsc::Sender;

/// A request to execute a query with its response channel
#[derive(Debug, Clone)]
pub struct QueryRequest {
    pub query: String,
    pub params: Vec<(String, Value)>,
    pub priority: QueryPriority,
    pub query_type: QueryType,
    pub table_name: Option<String>,
}

#[derive(Error, Debug, Clone)]
pub enum ExecutorError {
    #[error("Query execution failed: {0}")]
    ExecutionError(String),
    #[error("Query timed out")]
    Timeout,
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Batch processing error: {0}")]
    BatchError(String),
    #[error("Channel error: {0}")]
    ChannelError(String),
    #[error("Circuit breaker open")]
    CircuitBreakerOpen,
}

/// Query priority levels for scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QueryPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Query type for scheduling decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Read,
    Write,
    Schema,
}
