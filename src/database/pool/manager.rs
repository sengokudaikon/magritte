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

    async fn auth(&self, db: &Surreal<Any>) -> std::result::Result<(), surrealdb::Error> {
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

impl managed::Manager for Manager {
    type Type = Arc<Surreal<Any>>;
    type Error = surrealdb::Error;

    async fn create(&self) -> std::result::Result<Self::Type, Self::Error> {
        let db = any::connect(self.host.as_str()).await?;
        db.use_ns(self.ns.clone()).use_db(self.db.clone()).await?;
        self.auth(&db).await?;
        Ok(Arc::new(db))
    }

    async fn recycle(
        &self,
        conn: &mut Self::Type,
        _: &managed::Metrics,
    ) -> managed::RecycleResult<Self::Error> {
        conn.use_ns(self.ns.clone()).use_db(self.db.clone()).await?;
        self.auth(conn).await?;
        let _ = conn.query("INFO FOR DB;").await?;
        Ok(())
    }
}

pub type Pool = managed::Pool<Manager>;
