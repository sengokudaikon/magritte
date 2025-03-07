use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use crate::database::executor::{
    utils::metrics::ExecutorMetrics,
};
use serde_json::Value;
use crate::database::executor::core::types::{ExecutorError, QueryRequest};

pub mod config;
pub mod types;

/// Base executor trait for runtime-agnostic operations
#[async_trait]
pub trait BaseExecutor: Send + Sync {
    /// Start the executor
    async fn start(&self) -> Result<(), ExecutorError>;
    
    /// Stop the executor gracefully
    async fn stop(&self) -> Result<(), ExecutorError>;
    
    /// Get executor metrics
    async fn metrics(&self) -> Arc<ExecutorMetrics>;
    
    /// Execute a raw query and return deserialized results
    async fn execute(&self, request: QueryRequest) -> Result<Value, ExecutorError>;
    
    /// Check if the executor is healthy
    async fn is_healthy(&self) -> bool;
    
    /// Get the current executor state
    async fn state(&self) -> ExecutorState;
}

/// Represents the current state of an executor
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutorState {
    /// Executor is starting up
    Starting,
    /// Executor is running normally
    Running,
    /// Executor is shutting down
    ShuttingDown,
    /// Executor has stopped
    Stopped,
    /// Executor is in an error state
    Error(String),
} 