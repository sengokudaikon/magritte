use crate::database::executor::core::config::BatchConfig;
use crate::database::executor::core::types::{ExecutorError, QueryRequest, QueryType};
use dashmap::DashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// A batch of queries grouped by type and table
#[derive(Debug)]
pub struct QueryBatch {
    read_batch: RwLock<Vec<QueryRequest>>,
    write_batches: DashMap<String, Vec<QueryRequest>>,
    schema_batch: RwLock<Vec<QueryRequest>>,
    last_flush: RwLock<Instant>,
    config: BatchConfig,
}

impl QueryBatch {
    pub fn new(config: BatchConfig) -> Self {
        Self {
            read_batch: RwLock::new(Vec::new()),
            write_batches: DashMap::new(),
            schema_batch: RwLock::new(Vec::new()),
            last_flush: RwLock::new(Instant::now()),
            config,
        }
    }

    /// Add an incoming request to its corresponding batch
    pub async fn add_request(&self, request: QueryRequest) -> Result<(), ExecutorError> {
        match request.query_type {
            QueryType::Read => {
                let mut read_batch = self.read_batch.write().await;
                if read_batch.len() >= self.config.max_read_batch_size {
                    return Err(ExecutorError::BatchError("Read batch is full".into()));
                }
                read_batch.push(request);
            }
            QueryType::Write => {
                let key = request
                    .table_name
                    .clone()
                    .unwrap_or_else(|| "default".into());
                let mut batch = self.write_batches.entry(key).or_insert_with(Vec::new);
                if batch.len() >= self.config.max_write_batch_size {
                    return Err(ExecutorError::BatchError("Write batch is full".into()));
                }
                batch.push(request);
            }
            QueryType::Schema => {
                let mut schema_batch = self.schema_batch.write().await;
                if schema_batch.len() >= self.config.max_write_batch_size {
                    return Err(ExecutorError::BatchError("Schema batch is full".into()));
                }
                schema_batch.push(request);
            }
        }
        Ok(())
    }

    /// Returns true if any batch is full or the timeout elapsed
    pub async fn should_flush(&self) -> bool {
        let now = Instant::now();
        let last_flush = self.last_flush.read().await;
        if now.duration_since(*last_flush) >= self.config.batch_timeout {
            return true;
        }

        let read_batch = self.read_batch.read().await;
        if read_batch.len() >= self.config.max_read_batch_size {
            return true;
        }

        let schema_batch = self.schema_batch.read().await;
        if schema_batch.len() >= self.config.max_write_batch_size {
            return true;
        }

        for entry in self.write_batches.iter() {
            if entry.value().len() >= self.config.max_write_batch_size {
                return true;
            }
        }
        false
    }

    /// Flushes all batches and resets the timestamp
    pub async fn flush(&self) -> BatchResult {
        *self.last_flush.write().await = Instant::now();

        let reads = {
            let mut read_batch = self.read_batch.write().await;
            std::mem::take(&mut *read_batch)
        };

        let writes = {
            let mut writes = DashMap::new();
            // Take ownership of entries from write_batches
            let keys: Vec<String> = self
                .write_batches
                .iter()
                .map(|entry| entry.key().clone())
                .collect();
            for key in keys {
                if let Some((_, mut value)) = self.write_batches.remove(&key) {
                    writes.insert(key, value);
                }
            }
            writes
        };

        let schema = {
            let mut schema_batch = self.schema_batch.write().await;
            std::mem::take(&mut *schema_batch)
        };

        BatchResult {
            reads,
            writes,
            schema,
        }
    }
}

/// Result of a batch flush operation
#[derive(Debug)]
pub struct BatchResult {
    pub reads: Vec<QueryRequest>,
    pub writes: DashMap<String, Vec<QueryRequest>>,
    pub schema: Vec<QueryRequest>,
}
