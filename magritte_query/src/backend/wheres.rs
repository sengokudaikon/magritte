use crate::backend::value::SqlValue;
use crate::backend::QueryBuilder;
use crate::conditions::Operator;
use crate::expr::{HasConditions, HasParams};
use crate::types::TableType;
use crate::Callable;
use anyhow::anyhow;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::instrument;

pub trait WhereClause {
    fn where_op<V: Serialize>(
        self,
        field: &str,
        op: Operator,
        value: Option<V>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn where_in<U, QB: QueryBuilder<U>>(self, field: &str, subquery: QB) -> anyhow::Result<Self>
    where
        Self: Sized,
        U: TableType + Serialize + DeserializeOwned,;
    fn where_function<F: Callable>(self, func: F) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl<T: HasConditions + HasParams> WhereClause for T {
    /// Add a WHERE condition with an operator
    #[instrument(skip(self, value))]
    fn where_op<V: Serialize>(
        mut self,
        field: &str,
        op: Operator,
        value: Option<V>,
    ) -> anyhow::Result<Self> {
        if let Some(value) = value {
            let len = self.params_mut().len();
            let param_name = format!("p{}", len);
            self.params_mut()
                .push((param_name.clone(), serde_json::to_value(value)?));
            self.conditions_mut()
                .push((field.to_string(), op, SqlValue::Param(param_name)));
        } else {
            self.conditions_mut()
                .push((field.to_string(), op, SqlValue::Null));
        }
        Ok(self)
    }

    /// Add a WHERE IN subquery clause
    #[instrument(skip_all)]
    fn where_in<U, QB: QueryBuilder<U>>(mut self, field: &str, subquery: QB) -> anyhow::Result<Self>
    where
        U: TableType + Serialize + DeserializeOwned,
    {
        let subquery_str = format!("{} IN ({})", field, subquery.build()?);
        self.conditions_mut()
            .push((subquery_str, Operator::Raw, SqlValue::Null));
        Ok(self)
    }

    /// Add a WHERE condition with a function
    #[instrument(skip_all)]
    fn where_function<F: Callable>(mut self, func: F) -> anyhow::Result<Self> {
        if !func.can_filter() {
            return Err(anyhow!("Function {} cannot be used in WHERE clause", func));
        }
        self.conditions_mut()
            .push((func.to_string(), Operator::Raw, SqlValue::Null));
        Ok(self)
    }
}
