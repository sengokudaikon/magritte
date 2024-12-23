
use std::sync::Arc;
use surrealdb::engine::any::{connect, Any};
use surrealdb::Surreal;
#[cfg(test)]
pub async fn test_db() -> anyhow::Result<Arc<Surreal<Any>>> {
    let db = connect("mem://").await?;
    db.use_ns("test").use_db("test").await?;
    Ok(Arc::new(db))
}