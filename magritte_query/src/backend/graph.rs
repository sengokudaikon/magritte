use crate::{HasProjections, SelectStatement};
use magritte_core::operator::Operator;
use magritte_core::value::SqlValue;
use magritte_core::{Projection, RecursiveDepth, Relation, RelationDirection, TableType};
use serde::Serialize;

/// Trait for graph traversal operations
pub trait GraphTraversal {
    /// Add a graph traversal step
    fn relate(self, rel_type: RelationDirection, edge: &str, target: &str) -> Self;
    /// Add recursive traversal
    fn recursive(self, depth: RecursiveDepth) -> Self;
    fn with_alias(self, alias: &str) -> Self;

    /// Add return fields from edges [field1, field2]
    fn return_fields(self, fields: Vec<&str>) -> Self;

    /// Add conditions on edges (WHERE clause)
    fn with_edge_condition<V: Serialize>(self, field: &str, op: Operator, value: V) -> Self;

    /// Add subquery in edge conditions
    fn with_edge_subquery<U: TableType>(self, subquery: SelectStatement<U>) -> Self;

    /// Enable parallel processing
    fn parallel(self) -> Self;
}

impl<T: HasProjections> GraphTraversal for T {
    fn relate(mut self, rel_type: RelationDirection, edge: &str, target: &str) -> Self {
        self.projections_mut()
            .push(Projection::Relation(Relation::new(rel_type, edge, target)));
        self
    }

    fn recursive(mut self, depth: RecursiveDepth) -> Self {
        if let Some(Projection::Relation(relation)) = self.projections_mut().last_mut() {
            relation.recursive = Some(depth);
        }
        self
    }

    fn with_alias(mut self, alias: &str) -> Self {
        if let Some(Projection::Relation(relation)) = self.projections_mut().last_mut() {
            relation.alias = Some(String::from(alias))
        }
        self
    }

    fn return_fields(mut self, fields: Vec<&str>) -> Self {
        if let Some(Projection::Relation(relation)) = self.projections_mut().last_mut() {
            relation.return_fields = fields.into_iter().map(String::from).collect();
        }
        self
    }

    fn with_edge_condition<V: Serialize>(mut self, field: &str, op: Operator, value: V) -> Self {
        let rel_count = &self.projections().len();
        if let Some(Projection::Relation(relation)) = self.projections_mut().last_mut() {
            let param_len = relation.parameters.len();
            let param_name = format!("r{}p{}", rel_count, param_len,);
            let param_value = serde_json::to_value(value).expect("Failed to serialize value");
            let condition = (field.to_string(), op, SqlValue::Param(param_name.clone()));

            relation.parameters.push((param_name, param_value));
            relation.conditions.push(condition);
        }
        self
    }

    fn with_edge_subquery<U: TableType>(mut self, subquery: SelectStatement<U>) -> Self {
        if let Some(Projection::Relation(relation)) = self.projections_mut().last_mut() {
            relation.subquery = Some(subquery.build().unwrap_or_default());
        }
        self
    }

    fn parallel(mut self) -> Self {
        if let Some(Projection::Relation(relation)) = self.projections_mut().last_mut() {
            relation.parallel = true;
        }
        self
    }
}
