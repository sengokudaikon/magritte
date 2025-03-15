use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Configuration for query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorConfig {
    /// Connection pool configuration
    pub pool: PoolConfig,
    /// Query execution configuration
    pub query: QueryConfig,
    /// Batching configuration
    pub batch: BatchConfig,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Minimum number of connections to maintain
    pub min_connections: usize,
    /// Maximum number of connections allowed
    pub max_connections: usize,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Connection idle timeout
    pub idle_timeout: Duration,
    /// Maximum connection lifetime
    pub max_lifetime: Duration,
}

/// Query execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConfig {
    /// Query timeout
    pub query_timeout: Duration,
    /// Maximum retries per query
    pub max_retries: u32,
    /// Retry backoff base duration
    pub retry_backoff: Duration,
    /// Maximum concurrent queries
    pub max_concurrent_queries: usize,
}

/// Batching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Maximum batch size for read operations
    pub max_read_batch_size: usize,
    /// Maximum batch size for write operations
    pub max_write_batch_size: usize,
    /// Batch collection timeout
    pub batch_timeout: Duration,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            pool: PoolConfig::default(),
            query: QueryConfig::default(),
            batch: BatchConfig::default(),
        }
    }
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 5,
            max_connections: 32,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(3600),
        }
    }
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            query_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_backoff: Duration::from_millis(100),
            max_concurrent_queries: 100,
        }
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_read_batch_size: 1000,
            max_write_batch_size: 100,
            batch_timeout: Duration::from_millis(50),
        }
    }
}

impl ExecutorConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.pool.min_connections > self.pool.max_connections {
            return Err("min_connections cannot be greater than max_connections".into());
        }
        if self.batch.max_write_batch_size > self.batch.max_read_batch_size {
            return Err("max_write_batch_size cannot be greater than max_read_batch_size".into());
        }
        Ok(())
    }
}
