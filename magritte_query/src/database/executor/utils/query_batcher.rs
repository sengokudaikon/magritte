use crate::database::executor::core::config::BatchConfig;
use crate::database::executor::core::types::QueryType;
use crate::database::executor::{ExecutorError, QueryRequest};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::executor::core::types::QueryPriority;
    use tokio::sync::mpsc::channel;

    fn create_test_query(query_type: QueryType, table: Option<String>) -> QueryRequest {
        let (tx, _) = channel(1);
        QueryRequest {
            query: "test query".into(),
            params: vec![],
            priority: QueryPriority::Normal,
            query_type,
            table_name: table,
            response_tx: tx,
        }
    }

    #[tokio::test]
    async fn test_batch_limits() {
        let config = BatchConfig {
            max_read_batch_size: 2,
            max_write_batch_size: 2,
            batch_timeout: Duration::from_millis(50),
        };
        let batch = QueryBatch::new(config);

        // Test read batch
        assert!(batch
            .add_request(create_test_query(QueryType::Read, None))
            .await
            .is_ok());
        assert!(batch
            .add_request(create_test_query(QueryType::Read, None))
            .await
            .is_ok());
        assert!(batch
            .add_request(create_test_query(QueryType::Read, None))
            .await
            .is_err());

        // Test write batch
        let table = Some("test_table".into());
        assert!(batch
            .add_request(create_test_query(QueryType::Write, table.clone()))
            .await
            .is_ok());
        assert!(batch
            .add_request(create_test_query(QueryType::Write, table.clone()))
            .await
            .is_ok());
        assert!(batch
            .add_request(create_test_query(QueryType::Write, table))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_batch_flush() {
        let config = BatchConfig {
            max_read_batch_size: 10,
            max_write_batch_size: 10,
            batch_timeout: Duration::from_millis(50),
        };
        let batch = QueryBatch::new(config);

        // Add mixed queries
        batch
            .add_request(create_test_query(QueryType::Read, None))
            .await
            .unwrap();
        batch
            .add_request(create_test_query(QueryType::Write, Some("table1".into())))
            .await
            .unwrap();
        batch
            .add_request(create_test_query(QueryType::Schema, None))
            .await
            .unwrap();

        // Flush and verify
        let result = batch.flush().await;
        assert_eq!(result.reads.len(), 1);
        assert_eq!(result.writes.len(), 1);
        assert_eq!(result.schema.len(), 1);

        // Verify batches are empty after flush
        assert!(batch.read_batch.read().await.is_empty());
        assert!(batch.write_batches.is_empty());
        assert!(batch.schema_batch.read().await.is_empty());
    }
}
