use std::marker::PhantomData;
use crate::backend::QueryBuilder;
use crate::types::TableType;
use crate::SurrealDB;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
#[derive(Clone, Debug, PartialEq)]
pub struct UpsertStatement<T> {
    _marker: PhantomData<T>,
}

impl<T> UpsertStatement <T> where T: TableType + Serialize + DeserializeOwned {}
#[async_trait]
impl<T> QueryBuilder<T> for UpsertStatement<T>
where
    T: TableType + Serialize + DeserializeOwned,
{
    fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    fn build(&self) -> anyhow::Result<String> {
        todo!()
    }

    async fn execute(self, _conn: SurrealDB) -> anyhow::Result<Vec<T>> {
        todo!()
    }
}
