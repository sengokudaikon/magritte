use super::Result;
use crate::database::pool::credentials::Credentials;
use deadpool::managed;
use std::sync::Arc;
use surrealdb::{
    engine::{any, any::Any},
    opt::auth,
    Surreal,
};

pub struct Manager {
    host: String,
    ns: String,
    db: String,
    creds: Credentials,
}

impl Manager {
    pub fn new(host: String, ns: String, db: String, creds: Credentials) -> Self {
        Self {
            host,
            ns,
            db,
            creds,
        }
    }

    async fn auth(&self, db: &Surreal<Any>) -> Result<()> {
        match &self.creds {
            Credentials::Root { user, pass } => {
                db.signin(auth::Root {
                    username: user,
                    password: pass,
                })
                .await?;
            }
            Credentials::Namespace { user, pass, ns } => {
                db.signin(auth::Namespace {
                    username: user,
                    password: pass,
                    namespace: ns,
                })
                .await?;
            }
            Credentials::Database {
                user,
                pass,
                ns,
                db: database,
            } => {
                db.signin(auth::Database {
                    username: user,
                    password: pass,
                    namespace: ns,
                    database,
                })
                .await?;
            }
        }
        db.use_ns(&self.ns).use_db(&self.db).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl managed::Manager for Manager {
    type Error = surrealdb::Error;
    type Type = Arc<Surreal<Any>>;

    async fn create(&self) -> Result<Self::Type> {
        let db = any::connect(self.host.as_str()).await?;
        self.auth(&db).await?;
        Ok(Arc::new(db))
    }

    async fn recycle(
        &self,
        conn: &mut Self::Type,
        _: &managed::Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        // Re-authenticate
        self.auth(conn).await.map_err(|e| managed::RecycleError::from(e));

        // Perform a health check query
        // `INFO FOR DB;` is a simple SurrealDB statement that should return info about the current DB
        let _ = conn.query("INFO FOR DB;").await?;
        Ok(())
    }
}

pub type Pool = managed::Pool<Manager>;
