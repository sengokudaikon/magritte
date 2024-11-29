use async_trait::async_trait;
use crate::{QueryBuilder, SurrealDB};

#[derive(Clone, Debug)]
pub struct DefineStatement {}

impl DefineStatement {
    pub fn new() -> Self {
        Self {}
    }
}
#[async_trait]
impl<T> QueryBuilder<T> for DefineStatement {
    fn new() -> Self {
        todo!()
    }

    fn build(&self) -> anyhow::Result<String> {
        todo!()
    }

    async fn execute(self, _conn: SurrealDB) -> anyhow::Result<Vec<T>> {
        todo!()
    }
}