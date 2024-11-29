use crate::backend::QueryBuilder;
use crate::query_result::{FromTarget, QueryResult};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait FromClause {
    /// Add additional tables to query from
    fn from_targets_mut(&mut self) -> &mut Vec<FromTarget>;
    fn from_tables(mut self, tables: Vec<&str>) -> Self
    where
        Self: Sized,
    {
        self.from_targets_mut()
            .extend(tables.into_iter().map(|t| FromTarget::Table(t.to_string())));
        self
    }

    /// Add specific records to query from
    fn from_records(mut self, records: Vec<(&str, &str)>) -> Self
    where
        Self: Sized,
    {
        self.from_targets_mut().extend(
            records
                .into_iter()
                .map(|(table, id)| FromTarget::Record(table.to_string(), id.to_string())),
        );
        self
    }

    /// Add a list of record IDs to query from
    fn from_record_list(mut self, records: Vec<&str>) -> Self
    where
        Self: Sized,
    {
        self.from_targets_mut().push(FromTarget::RecordList(
            records.into_iter().map(String::from).collect(),
        ));
        self
    }

    /// Add a FROM target that is a subquery
    fn from_subquery<U, QB: QueryBuilder<U>>(mut self, subquery: QB) -> anyhow::Result<Self>
    where
        U: Clone + Send + Sync + 'static + Serialize + DeserializeOwned,
        Self: Sized,
    {
        self.from_targets_mut()
            .push(FromTarget::Subquery(QueryResult::Raw(subquery.build()?)));
        Ok(self)
    }

    /// Add a dynamic Table reference
    fn from_dynamic(mut self, expr: &str) -> Self
    where
        Self: Sized,
    {
        self.from_targets_mut()
            .push(FromTarget::Dynamic(expr.to_string()));
        self
    }
}
