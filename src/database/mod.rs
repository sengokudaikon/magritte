mod pool;
mod executor;
mod runtime;
mod scheduler;
mod rw;

use std::sync::Arc;
use anyhow::Result;
use async_channel::{bounded, Sender};
use serde::de::DeserializeOwned;
use crate::database::executor::{Executor, QueryRequest};
use crate::database::pool::config::DbConfig;
use crate::database::pool::connection::SurrealConnectionManager;
use crate::query::Query;

pub struct Database {
    executor: Arc<dyn Executor>,
    config: DbConfig,
    query_tx: Sender<QueryRequest>,
}

impl Database {
    pub async fn new(config: DbConfig) -> Result<Arc<Self>> {
        let (query_tx, query_rx) = bounded(1024); // Bounded MPMC channel for query distribution
        
        // Create connection manager for tokio runtime
        let pool = SurrealConnectionManager::tokio(config.clone()).await?;
            
        // Create and start executor
        let executor = {
            #[cfg(feature = "tokio")]
            {
                let executor = Arc::new(executor::tokio_executor::TokioExecutor::new(
                    Arc::new(pool),
                    query_rx,
                    query_tx.clone(),
                ));
                
                // Start executor event loop
                let exec_clone = executor.clone();
                tokio::spawn(async move {
                    exec_clone.run().await?;
                    Ok::<_, anyhow::Error>(())
                });
                
                executor
            }
            
            #[cfg(feature = "async_std")]
            {
                let executor = Arc::new(executor::async_std_executor::AsyncStdExecutor::new(
                    Arc::new(pool),
                    query_rx,
                    query_tx.clone(),
                ));
                
                // Start executor event loop
                let exec_clone = executor.clone();
                async_std::task::spawn(async move {
                    exec_clone.run().await?;
                    Ok::<_, anyhow::Error>(())
                });
                
                executor
            }
            
            #[cfg(not(any(feature = "tokio", feature = "async_std")))]
            {
                let executor = Arc::new(executor::tokio_executor::TokioExecutor::new(
                    Arc::new(pool),
                    query_rx,
                    query_tx.clone(),
                ));
                
                // Start executor event loop
                let exec_clone = executor.clone();
                tokio::spawn(async move {
                    exec_clone.run().await?;
                    Ok::<_, anyhow::Error>(())
                });
                
                executor
            }
        };

        Ok(Arc::new(Self {
            executor,
            config,
            query_tx,
        }))
    }

    pub async fn execute<T>(&self, query: Query) -> Result<Vec<T>> 
    where 
        T: DeserializeOwned + Send + 'static 
    {
        // Build query and extract parameters
        let query_str = query.build()?;
        let params = query.get_params().to_vec();
        
        // Execute via executor
        self.executor.execute(query_str, params).await
    }
}
