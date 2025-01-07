use anyhow::anyhow;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use tracing::error;
use magritte_query::Query;
use crate::database::executor::Executor;
use crate::database::pool::connection::{SurrealConnection, SurrealConnectionManager};

#[derive(Clone)]
pub struct TokioExecutor {
    manager: SurrealConnectionManager,
}

impl TokioExecutor {
    pub fn new(manager: SurrealConnectionManager) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl Executor for TokioExecutor

{
    async fn execute<T: Serialize + DeserializeOwned>(&self, query: &Query) -> anyhow::Result<Vec<T>> {
        // 1. get a connection
        let conn = self.manager.get_conn().await?;
        let sql = query.to_string(); // or .build(), or something
        let mut surreal_query = conn.query(query);
        // 3. run it
        let res = surreal_query.await?.take(0);
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("Query execution failed: {:?}", e);
                Err(anyhow!(e))
            }
        }
    }
}