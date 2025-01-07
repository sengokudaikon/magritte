use super::{PoolError, Result};
use crate::database::pool::credentials::Credentials;
use crate::database::pool::manager::{Manager, Pool};
use deadpool::Runtime;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use surrealdb::{engine::any::Any, Surreal};
use crate::database::pool::config::DbConfig;

#[async_trait::async_trait]
pub trait SurrealConnection {
    async fn get_conn(&self) -> Result<Arc<Surreal<Any>>>;
}

/// Builder for `SurrealConnectionManager`
pub struct SurrealConnectionManagerBuilder {
    host: Option<String>,
    ns: Option<String>,
    db: Option<String>,
    creds: Option<Credentials>,
    max_size: usize,
    connect_timeout: u64,
    idle_timeout: u64,
}

impl SurrealConnectionManagerBuilder {
    pub fn new() -> Self {
        Self {
            host: None,
            ns: None,
            db: None,
            creds: None,
            max_size: 5,
            connect_timeout: 5,
            idle_timeout: 30,
        }
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }

    pub fn namespace(mut self, ns: &str) -> Self {
        self.ns = Some(ns.to_string());
        self
    }

    pub fn database(mut self, db: &str) -> Self {
        self.db = Some(db.to_string());
        self
    }

    pub fn credentials(mut self, creds: Credentials) -> Self {
        self.creds = Some(creds);
        self
    }

    pub fn max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }

    pub fn connect_timeout(mut self, secs: u64) -> Self {
        self.connect_timeout = secs;
        self
    }

    pub fn idle_timeout(mut self, secs: u64) -> Self {
        self.idle_timeout = secs;
        self
    }

    pub async fn with_config(self, config: DbConfig, runtime: Runtime) -> Result<SurrealConnectionManager> {
        let creds = match config.credentials_type.as_str() {
            "root" => {
                Credentials::Root {
                    user: config.user.clone().unwrap_or_default(),
                    pass: config.pass.clone().unwrap_or_default(),
                }
            }
            "namespace" => {
                Credentials::Namespace {
                    user: config.user.clone().unwrap_or_default(),
                    pass: config.pass.clone().unwrap_or_default(),
                    ns: config.ns.clone(),
                }
            }
            "database" => {
                Credentials::Database {
                    user: config.user.clone().unwrap_or_default(),
                    pass: config.pass.clone().unwrap_or_default(),
                    ns: config.ns.clone(),
                    db: config.db.clone(),
                }
            }
            _ => panic!("Database is misconfigured"),
        };
        let manager = Manager::new(config.url.clone(), config.ns.clone(), config.db.clone(), creds);
        let pool = Pool::builder(manager)
            .max_size(config.max_connections)
            .wait_timeout(Some(Duration::from_secs(config.connect_timeout)))
            .create_timeout(Some(Duration::from_secs(config.connect_timeout)))
            .recycle_timeout(Option::from(Duration::from_secs(config.idle_timeout)))
            .runtime(runtime)
            .build().map_err(PoolError::from)?;

        Ok(SurrealConnectionManager { pool })
    }

    pub async fn build(self) -> Result<SurrealConnectionManager> {
        let host = self
            .host
            .ok_or_else(|| PoolError::Config("Host is required".to_string()))?;
        let ns = self
            .ns
            .ok_or_else(|| PoolError::Config("Namespace is required".to_string()))?;
        let db = self
            .db
            .ok_or_else(|| PoolError::Config("Database is required".to_string()))?;
        let creds = self
            .creds
            .ok_or_else(|| PoolError::Config("Credentials are required".to_string()))?;

        let manager = Manager::new(host, ns, db, creds);
        let pool = Pool::builder(manager)
            .max_size(self.max_size)
            .wait_timeout(Some(Duration::from_secs(self.connect_timeout)))
            .create_timeout(Some(Duration::from_secs(self.connect_timeout)))
            .recycle_timeout(Some(Duration::from_secs(self.idle_timeout)))
            .runtime(Runtime::Tokio1)
            .build()
            .map_err(|e| PoolError::Generic(format!("Failed to create pool: {}", e)))?;

        // Validate the pool by getting a connection and checking health
        let conn = pool
            .get()
            .await
            .map_err(|e| PoolError::Generic(format!("Failed to get connection: {}", e)))?;
        // Health check: we already do this in recycle, but let's do a one-time check here too.
        conn.query("INFO FOR DB;").await.map_err(PoolError::Surreal)?;

        Ok(SurrealConnectionManager { pool })
    }
}

#[derive(Clone)]
pub struct SurrealConnectionManager {
    pool: Pool,
}

#[async_trait::async_trait]
impl SurrealConnection for SurrealConnectionManager {
    async fn get_conn(&self) -> Result<Arc<Surreal<Any>>> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| PoolError::Generic(format!("Failed to get connection: {}", e)))?;
        Ok(conn.deref().clone())
    }
}

#[async_trait::async_trait]
impl SurrealConnection for Arc<SurrealConnectionManager> {
    async fn get_conn(&self) -> Result<Arc<Surreal<Any>>> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| PoolError::Generic(format!("Failed to get connection: {}", e)))?;
        Ok(conn.deref().clone())
    }
}

impl SurrealConnectionManager {
    pub async fn default(config: DbConfig) -> Result<Self> {
        SurrealConnectionManager::builder()
            .with_config(config, Runtime::Tokio1).await
    }

    pub async fn tokio(config: DbConfig) -> Result<Self> {
        Self::default(config).await
    }
    pub async fn async_std(config: DbConfig) -> Result<Self> {
        SurrealConnectionManager::builder()
            .with_config(config, Runtime::AsyncStd1)
            .await
    }
    pub async fn get(&self) -> Result<Arc<Surreal<Any>>> {
        self.get_conn().await
    }

    pub fn builder() -> SurrealConnectionManagerBuilder {
        SurrealConnectionManagerBuilder::new()
    }

    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    pub async fn close(&self) {
        self.pool.close();
    }
}
