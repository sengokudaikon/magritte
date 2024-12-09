use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization/Deserialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Generic error: {0}")]
    Generic(String),

    #[error("External error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("SurrealDB error: {0}")]
    SurrealDB(#[from] surrealdb::Error),
}

pub type Result<T> = std::result::Result<T, Error>;