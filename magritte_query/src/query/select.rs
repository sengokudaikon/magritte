//! SELECT query operations
//!
//! This module contains operations related to selecting and retrieving data
//! from tables.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{map, Value};
use tracing::{error, info, instrument};

use crate::backend::value::SqlValue;
use crate::backend::QueryBuilder;
use crate::conditions::Operator;
use crate::define::index::Indexable;
use crate::expr::{HasConditions, HasLetConditions, HasParams, HasRelations, HasVectorConditions};
use crate::func::{Callable, CanCallFunctions, CountFunction};
use crate::graph::Relation;
use crate::query_result::{FromTarget, QueryResult};
use crate::types::{OrderBy, RangeTarget, RecordType, SurrealId, TableType};
use crate::vector_search::{VectorCondition, VectorSearch};
use crate::SurrealDB;

#[derive(Clone, Debug, PartialEq)]
pub struct SelectStatement<T> where T:RecordType{
    pub(crate) targets: Option<Vec<FromTarget<T>>>,
    pub(crate) select_value: bool,
    pub(crate) with_id: Option<SurrealId<T>>,
    pub(crate) only: bool,
    pub(crate) selected_fields: Option<Vec<QueryResult>>,
    pub(crate) omitted_fields: Option<Vec<String>>,
    pub(crate) conditions: Vec<(String, Operator, SqlValue)>,
    pub(crate) order_by: Vec<(OrderBy, bool)>,
    pub(crate) group_by: Vec<String>,
    pub(crate) all: bool,
    pub(crate) limit: Option<usize>,
    pub(crate) start: Option<String>,
    pub(crate) parameters: Vec<(String, Value)>,
    pub(crate) split_fields: Vec<String>,
    pub(crate) fetch_fields: Vec<String>,
    pub(crate) relations: Vec<Relation>,
    pub(crate) parallel: bool,
    pub(crate) with_index: Option<Vec<String>>,
    pub(crate) tempfiles: bool,
    pub(crate) timeout: Option<Duration>,
    pub(crate) vector_conditions: Vec<VectorCondition>,
    pub(crate) explain: Option<bool>,
    pub(crate) version: Option<String>,
    pub(crate) let_statements: Vec<(String, String)>,
    phantom_data: PhantomData<T>
}

// Base implementation for all states
impl<T> SelectStatement<T>
where
    T: RecordType
{
    /// Select specific fields, optionally with aliases
    #[instrument(skip(self))]
    pub fn field(mut self, expr: &str, alias: Option<&str>) -> Self {
        let fields = self.selected_fields.get_or_insert_with(Vec::new);
        fields.push(QueryResult::Field {
            expr: expr.to_string(),
            alias: alias.map(String::from),
        });
        self
    }
    pub fn where_id(mut self, id: SurrealId<T>) -> Self {
        self.with_id = Some(id);
        self
    }

    /// Select multiple fields
    #[instrument(skip(self))]
    pub fn fields(mut self, fields: &[&str]) -> Self {
        let l_fields = self.selected_fields.get_or_insert_with(Vec::new);
        for expr in fields.iter() {
            l_fields.push(QueryResult::Field {
                expr: expr.to_string(),
                alias: None,
            });
        }
        self
    }

    /// Select multiple fields
    #[instrument(skip(self))]
    pub fn fields_with_alias(mut self, fields: &[(&str, Option<&str>)]) -> Self {
        let l_fields = self.selected_fields.get_or_insert_with(Vec::new);
        for (expr, alias) in fields.iter() {
            l_fields.push(QueryResult::Field {
                expr: expr.to_string(),
                alias: alias.map(String::from),
            });
        }
        self
    }

    /// Add a subquery to SELECT fields
    #[instrument(skip_all)]
    pub fn subquery<U, QB: QueryBuilder<U>>(
        mut self,
        subquery: QB,
        alias: Option<&str>,
    ) -> Result<Self>
    where
        U: RecordType
    {
        let fields = self.selected_fields.get_or_insert_with(Vec::new);
        fields.push(QueryResult::Subquery {
            query: subquery.build()?,
            alias: alias.map(String::from),
        });
        Ok(self)
    }

    /// Select fields with raw expressions
    #[instrument(skip(self))]
    pub fn raw(mut self, raw_sql: &str) -> Self {
        let fields = self.selected_fields.get_or_insert_with(Vec::new);
        fields.push(QueryResult::Raw(raw_sql.into()));
        self
    }

    /// Add ORDER BY clause
    #[instrument(skip(self))]
    pub fn order_by_field(mut self, field: &str, ascending: bool) -> Self {
        self.order_by
            .push((OrderBy::Field(field.to_string()), ascending));
        self
    }

    /// Order results randomly
    #[instrument(skip(self))]
    pub fn order_by_rand(mut self) -> Self {
        self.order_by.push((OrderBy::Random, true));
        self
    }

    /// Order results with collation
    #[instrument(skip(self))]
    pub fn order_by_collate(mut self, field: &str, ascending: bool) -> Self {
        self.order_by
            .push((OrderBy::Collate(field.to_string()), ascending));
        self
    }

    /// Order results numerically
    #[instrument(skip(self))]
    pub fn order_by_numeric(mut self, field: &str, ascending: bool) -> Self {
        self.order_by
            .push((OrderBy::Numeric(field.to_string()), ascending));
        self
    }

    /// Add GROUP BY clause
    #[instrument(skip(self))]
    pub fn group_by(mut self, field: &str) -> Self {
        self.group_by.push(field.to_string());
        self
    }

    pub fn group_all(mut self) -> Self {
        self.all = true;
        self
    }

    /// Add START clause
    #[instrument(skip(self))]
    pub fn start(mut self, start: &str) -> Self {
        self.start = Some(start.to_string());
        self
    }

    /// Select VALUE instead of fields
    #[instrument(skip(self))]
    pub fn select_value(mut self) -> Self {
        self.select_value = true;
        self
    }

    /// Omit specific fields from the result
    #[instrument(skip(self))]
    pub fn omit(mut self, fields: Vec<&str>) -> Self {
        self.omitted_fields = Some(fields.into_iter().map(String::from).collect());
        self
    }

    /// Add WITH INDEX|NOINDEX if empty vec
    #[instrument(skip(self))]
    pub fn with_indexes(mut self, indexes: Vec<String>) -> Self {
        self.with_index = Some(indexes);
        self
    }

    /// Convert to ONLY state - requires subsequent limit call
    pub fn only(mut self) -> Self {
        self.only = true;
        self
    }

    /// Limit results by [usize] - required for ONLY statements (will produce a
    /// runtime error otherwise)
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Simple count, returns 1
    pub fn count(mut self) -> Self {
        self.selected_fields
            .get_or_insert_with(Vec::new)
            .push(QueryResult::Raw(CountFunction::Count.to_string()));
        self
    }

    /// Count truthy value
    pub fn count_value(mut self, value: &str) -> Self {
        self.selected_fields
            .get_or_insert_with(Vec::new)
            .push(QueryResult::Raw(
                CountFunction::CountValue(value.to_string()).to_string(),
            ));
        self
    }

    /// Count truthy values in array
    pub fn count_array(mut self, array: &str) -> Self {
        self.selected_fields
            .get_or_insert_with(Vec::new)
            .push(QueryResult::Raw(
                CountFunction::CountArray(array.to_string()).to_string(),
            ));
        self
    }

    /// Split results by array field values
    ///
    /// This will return a separate record for each value in the specified array
    /// field.
    ///
    /// # Example
    /// ```rust,ignore
    /// # use query_builder::QB;
    /// let query = QB::<User>::new(db)
    ///     .split("emails")  // For array field emails = ["a@b.com", "c@d.com"]
    ///     .build();
    /// // Will return two records, one for each email
    /// ```
    #[instrument(skip(self))]
    pub fn split(mut self, field: &str) -> Self {
        self.split_fields.push(field.to_string());
        self
    }

    /// Split results by multiple array fields
    #[instrument(skip(self))]
    pub fn split_fields(mut self, fields: Vec<&str>) -> Self {
        self.split_fields
            .extend(fields.into_iter().map(String::from));
        self
    }

    /// Enable query explanation
    #[instrument(skip(self))]
    pub fn explain(mut self, full: bool) -> Self {
        self.explain = Some(full);
        self
    }

    /// Add fields to fetch from related records
    #[instrument(skip(self))]
    pub fn fetch(mut self, fields: &[&str]) -> Self {
        self.fetch_fields
            .extend(fields.iter().map(|f| String::from(*f)));
        self
    }

    /// Add a TIMEOUT duration
    #[instrument(skip_all)]
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Enable parallel execution
    #[instrument(skip(self))]
    pub fn parallel(mut self) -> Self {
        self.parallel = true;
        self
    }

    /// Enable tempfiles for large datasets
    #[instrument(skip(self))]
    pub fn tempfiles(mut self) -> Self {
        self.tempfiles = true;
        self
    }

    /// Add VERSION clause
    pub fn version(mut self, timestamp: &str) -> Self {
        self.version = Some(format!("d'{}'", timestamp));
        self
    }

    /// Add a range to the target Table
    ///
    /// # Examples
    /// ```rust,ignore
    /// // Select all person records with IDs between 1 and 1000
    /// QB::<Person>::select(db).range("1", "1000");
    /// ```
    pub fn range(mut self, start: &str, end: &str) -> Self {
        if let Some(targets) = &mut self.targets {
            targets.push(FromTarget::Range(
                T::table_name().to_string(),
                RangeTarget::Range(start.to_string(), end.to_string()),
            ));
        } else {
            self.targets = Some(vec![FromTarget::Range(
                T::table_name().to_string(),
                RangeTarget::Range(start.to_string(), end.to_string()),
            )]);
        }
        self
    }

    /// Filter array values in a field
    ///
    /// # Examples
    /// ```rust,ignore
    /// // SELECT address[WHERE active = true] FROM person
    /// QB::<Person>::select(db)
    ///     .field_filter("address", "active = true");
    /// ```
    pub fn field_filter(mut self, field: &str, condition: &str) -> Result<Self> {
        let fields = self.selected_fields.get_or_insert_with(Vec::new);
        fields.push(QueryResult::Raw(format!("{}[WHERE {}]", field, condition)));
        Ok(self)
    }

    /// Filter array values with a parameterized condition
    pub fn field_filter_with_condition<V: Serialize>(
        mut self,
        field: &str,
        condition_field: &str,
        op: Operator,
        value: V,
    ) -> Result<Self> {
        let param_len = self.parameters.len();
        let param_name = format!("p{}", param_len);
        self.parameters
            .push((param_name.clone(), serde_json::to_value(value)?));

        let fields = self.selected_fields.get_or_insert_with(Vec::new);
        fields.push(QueryResult::Raw(format!(
            "{}[WHERE {} {} ${}]",
            field,
            condition_field,
            String::from(op),
            param_name
        )));
        Ok(self)
    }
}
impl<T> HasVectorConditions for SelectStatement<T>
where
    T: RecordType
{
    fn get_vector_conditions(&self) -> &Vec<VectorCondition> {
        &self.vector_conditions
    }

    fn get_vector_conditions_mut(&mut self) -> &mut Vec<VectorCondition> {
        &mut self.vector_conditions
    }
}

impl<T> HasParams for SelectStatement<T>
where
    T: RecordType
{
    fn params(&self) -> &Vec<(String, Value)> {
        &self.parameters
    }

    fn params_mut(&mut self) -> &mut Vec<(String, Value)> {
        &mut self.parameters
    }
}

impl<T> HasConditions for SelectStatement<T>
where
    T: RecordType
{
    fn conditions_mut(&mut self) -> &mut Vec<(String, Operator, SqlValue)> {
        &mut self.conditions
    }
}
impl<T> HasRelations for SelectStatement<T>
where
    T: RecordType
{
    fn relations(&self) -> &Vec<Relation> {
        &self.relations
    }

    fn relations_mut(&mut self) -> &mut Vec<Relation> {
        &mut self.relations
    }
}
impl<T> CanCallFunctions for SelectStatement<T>
where
    T: RecordType
{
    /// Call a standard function
    fn call_function<F: Callable>(mut self, func: F) -> Self {
        self.selected_fields
            .get_or_insert_with(Vec::new)
            .push(QueryResult::Raw(func.to_string()));
        self
    }
}

impl<T> Indexable for SelectStatement<T>
where
    T: RecordType
{
    fn with_index(&self) -> &Option<Vec<String>> {
        &self.with_index
    }

    fn with_index_mut(&mut self) -> &mut Option<Vec<String>> {
        &mut self.with_index
    }
}

impl<T> HasLetConditions for SelectStatement<T>
where
    T: RecordType
{
    fn get_lets(&self) -> &Vec<(String, String)> {
        &self.let_statements
    }

    fn get_lets_mut(&mut self) -> &mut Vec<(String, String)> {
        &mut self.let_statements
    }
}

#[async_trait]
impl<T> QueryBuilder<T> for SelectStatement<T>
where
    T: RecordType
{
    fn new() -> Self {
        Self {
            selected_fields: None,
            omitted_fields: None,
            conditions: Vec::new(),
            order_by: Vec::new(),
            group_by: Vec::new(),
            all: false,
            limit: None,
            fetch_fields: Vec::new(),
            relations: Vec::new(),
            with_index: None,
            parameters: Vec::new(),
            split_fields: Vec::new(),
            select_value: false,
            timeout: None,
            vector_conditions: Vec::new(),
            targets: None,
            only: false,
            start: None,
            parallel: false,
            tempfiles: false,
            explain: None,
            version: None,
            let_statements: vec![],
            phantom_data: PhantomData,
            with_id: None,
        }
    }

    fn build(&self) -> Result<String> {
        let mut query = String::new();
        let mut params = self.parameters.clone();
        if !self.let_statements.is_empty() {
            let statements = &self
                .let_statements
                .iter()
                .map(|(name, value)| format!("LET ${} = {};", name, value))
                .collect::<Vec<String>>();
            query.push_str(&statements.join(" "));
            query.push(' ')
        }
        query.push_str("SELECT ");
        // Add EXPLAIN if needed
        if let Some(full) = self.explain {
            query.push_str("EXPLAIN ");
            if full {
                query.push_str("FULL ");
            }
        }
        // Add VALUE if specified
        if self.select_value {
            query.push_str("VALUE ");
        }
        // Add fields
        match (&self.selected_fields, &self.omitted_fields) {
            (Some(fields), _) => {
                let field_strs: Vec<String> = fields.iter().map(|f| f.to_string()).collect();
                query.push_str(&field_strs.join(", "));
            }
            (None, Some(omit)) => {
                query.push_str("* OMIT ");
                query.push_str(&omit.join(", "));
            }
            (None, None) => query.push('*'),
        }
        // Add FROM clause
        query.push_str(" FROM ");
        if self.only {
            if self.limit.is_none() {
                bail!("When using .only(), a LIMIT 1 must be specified.");
            }
            query.push_str("ONLY ");
            if let Some(id) = &self.with_id {
                query.push_str(id.to_string().as_str());
            } else {
                query.push_str(T::table_name());
            }
        } else if let Some(targets) = &self.targets {
            if !targets.is_empty() {
                for v in targets {
                    query.push_str(&format!("{},", v.to_string().as_str()));
                }
                query = query.trim_end_matches(',').parse()?;
            }
        } else {
            if let Some(id) = &self.with_id {
                query.push_str(id.to_string().as_str());
            } else {
                query.push_str(T::table_name());
            }
        }

        // Add relations if present
        if !self.relations.is_empty() {
            for relation in &self.relations {
                query.push_str(&relation.build_query_part());
                // Add relation parameters to the query parameters
                params.extend(relation.parameters.clone());
            }
        }

        if let Some(with) = &self.with_index {
            if with.is_empty() {
                query.push_str(" NOINDEX")
            } else {
                query.push_str(" WITH INDEX ");
                query.push_str(&with.join(", "));
            }
        }

        // Add WHERE clause
        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            let conditions: Vec<String> = self
                .conditions
                .iter()
                .map(|(field, op, value)| {
                    format!("{} {} {}", field, String::from(op.clone()), value)
                })
                .collect();
            query.push_str(&conditions.join(" AND "));
        }

        // Add GROUP BY
        if !self.group_by.is_empty() {
            query.push_str(" GROUP BY ");
            query.push_str(&self.group_by.join(", "));
        } else if self.all {
            query.push_str(" GROUP ALL");
        }

        // Add ORDER BY
        if !self.order_by.is_empty() {
            query.push_str(" ORDER BY ");
            let orders: Vec<String> = self
                .order_by
                .iter()
                .map(|(field, asc)| {
                    let dir = if *asc { "ASC" } else { "DESC" };
                    match field {
                        OrderBy::Random => "rand()".to_string(),
                        OrderBy::Field(f) => format!("{} {}", f, dir),
                        OrderBy::Collate(f) => format!("{} COLLATE {}", f, dir),
                        OrderBy::Numeric(f) => format!("{} NUMERIC {}", f, dir),
                    }
                })
                .collect();
            query.push_str(&orders.join(", "));
        }

        // Add LIMIT and START AT
        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
            if let Some(offset) = &self.start {
                query.push_str(&format!(" START AT {}", offset));
            }
        }

        // Add SPLIT
        if !self.split_fields.is_empty() {
            query.push_str(" SPLIT ");
            query.push_str(&self.split_fields.join(", "));
        }

        // Add FETCH
        if !self.fetch_fields.is_empty() {
            query.push_str(" FETCH ");
            query.push_str(&self.fetch_fields.join(", "));
        }

        // Add TIMEOUT
        if let Some(timeout) = &self.timeout {
            query.push_str(&format!(" TIMEOUT {}", timeout.as_secs()));
        }

        // Add PARALLEL and TEMPFILES
        if self.parallel {
            query.push_str(" PARALLEL");
        }
        if self.tempfiles {
            query.push_str(" TEMPFILES");
        }

        // Add vector conditions
        if !self.vector_conditions.is_empty() {
            query.push_str(&Self::build_vector_conditions(&self.vector_conditions));
        }

        // Add VERSION
        if let Some(ver) = &self.version {
            query.push_str(&format!(" VERSION {}", ver));
        }

        query.push(';');
        Ok(query)
    }

    #[instrument(skip(self))]
    async fn execute(mut self, conn: SurrealDB) -> Result<Vec<T>> {
        let query = self.build()?;
        info!("Executing query: {}", query);

        let mut surreal_query = conn.query(query);

        // Bind all parameters
        for (name, value) in self.parameters {
            surreal_query = surreal_query.bind((name, value));
        }

        let res = surreal_query.await?.take(0);
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("Query execution failed: {:?}", e);
                Err(anyhow!(e))
            }
        }
    }
}
