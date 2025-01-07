use std::sync::Arc;
use anyhow::Result;
use async_channel::{bounded, Receiver, Sender};
use serde::de::DeserializeOwned;
use serde_json::Value;
use crate::database::pool::connection::SurrealConnection;
use crate::query::Query;

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

/// A scheduled query with metadata
#[derive(Debug)]
pub struct ScheduledQuery {
    pub query: String,
    pub params: Vec<(String, Value)>,
    pub priority: QueryPriority,
    pub query_type: QueryType,
    pub response_tx: Sender<Result<Value>>,
}

impl ScheduledQuery {
    pub fn new(
        query: String,
        params: Vec<(String, Value)>,
        priority: QueryPriority,
        query_type: QueryType,
        response_tx: Sender<Result<Value>>,
    ) -> Self {
        Self {
            query,
            params,
            priority,
            query_type,
            response_tx,
        }
    }
}

/// Scheduler configuration
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Maximum number of parallel read queries
    pub max_parallel_reads: usize,
    /// Maximum number of parallel write queries
    pub max_parallel_writes: usize,
    /// Queue size for pending queries
    pub queue_size: usize,
    /// Whether to use cooperative scheduling
    pub cooperative: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_parallel_reads: 32,
            max_parallel_writes: 8,
            queue_size: 1024,
            cooperative: true,
        }
    }
}

/// Core scheduler trait that handles query distribution
#[async_trait::async_trait]
pub trait Scheduler: Send + Sync + 'static {
    /// Schedule a query for execution
    async fn schedule(&self, query: ScheduledQuery) -> Result<()>;
    
    /// Start the scheduler
    async fn start(&self) -> Result<()>;
    
    /// Stop the scheduler
    async fn stop(&self) -> Result<()>;
    
    /// Get scheduler metrics
    async fn metrics(&self) -> SchedulerMetrics;
}

/// Scheduler metrics for monitoring
#[derive(Debug, Clone, Default)]
pub struct SchedulerMetrics {
    pub active_reads: usize,
    pub active_writes: usize,
    pub queued_reads: usize,
    pub queued_writes: usize,
    pub completed_reads: usize,
    pub completed_writes: usize,
    pub failed_reads: usize,
    pub failed_writes: usize,
} 