use std::future::Future;
use std::pin::Pin;

pub mod config;
pub mod connection;
pub mod credentials;
pub mod manager;
mod runtime;

pub type BoxedResultSendFuture<'r, T, E> =
    Pin<Box<dyn Future<Output = std::result::Result<T, E>> + 'r + Send>>;

use serde_json::Error as SerdeError;
use std::io;
use surrealdb::Error as SurrealError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PoolError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("SurrealDB error: {0}")]
    Surreal(#[from] SurrealError),

    #[error("JSON error: {0}")]
    Json(#[from] SerdeError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Generic error: {0}")]
    Generic(String),

    #[error("Build error: {0}")]
    Build(#[from] deadpool::managed::BuildError)
}

pub type Result<T> = std::result::Result<T, PoolError>;
