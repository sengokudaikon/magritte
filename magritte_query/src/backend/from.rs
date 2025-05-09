use crate::query_result::{FromTarget, QueryResult};
use crate::SelectStatement;
use magritte_core::{RecordType, SurrealId};

pub trait FromClause<T>
where
    T: RecordType,
{
    /// Add additional tables to query from
    fn from_targets_mut(&mut self) -> &mut Vec<FromTarget<T>>;
    fn from_tables(mut self, tables: Vec<&str>) -> Self
    where
        Self: Sized,
    {
        self.from_targets_mut()
            .extend(tables.into_iter().map(|t| FromTarget::Table(t.to_string())));
        self
    }
    fn from_record(mut self, target: SurrealId<T>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        self.from_targets_mut().push(FromTarget::Record(target));
        Ok(self)
    }

    /// Add a list of record IDs to query from
    fn from_records(mut self, targets: Vec<SurrealId<T>>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let targets = vec![FromTarget::RecordList(targets)];
        self.from_targets_mut().extend(targets);
        Ok(self)
    }

    /// Add a FROM target that is a subquery
    fn from_subquery(mut self, subquery: SelectStatement<T>) -> anyhow::Result<Self>
    where
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
