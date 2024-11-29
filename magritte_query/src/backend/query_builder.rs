use super::Result;
use crate::SurrealDB;
use async_trait::async_trait;

#[async_trait]
pub trait QueryBuilder<T> {
    fn new() -> Self;
    fn build(&self) -> Result<String>;
    async fn execute(mut self, conn: SurrealDB) -> Result<Vec<T>>;
}
