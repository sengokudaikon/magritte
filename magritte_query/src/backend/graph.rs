use std::fmt::{Display, Formatter};
use serde::Serialize;
use serde_json::Value;

use super::conditions::Operator;
use super::types::Projection;
use crate::backend::value::SqlValue;
use crate::expr::HasProjections;
use crate::SelectStatement;
use crate::types::TableType;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelationDirection {
    In,   // <-
    Out,  // ->
    Both, // <->
}

impl Display for RelationDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationDirection::In => write!(f, "<-"),
            RelationDirection::Out => write!(f, "->"),
            RelationDirection::Both => write!(f, "<->"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecursiveDepth {
    Fixed(usize),             // @.{3}
    Range(usize, usize),      // @.{1..5}
    OpenEnded(Option<usize>), // @.{..} or @.{..256}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relation {
    pub direction: RelationDirection,
    pub edge: String,
    pub target: String,
    pub recursive: Option<RecursiveDepth>,
    pub alias: Option<String>,
    pub return_fields: Vec<String>,
    pub conditions: Vec<(String, Operator, SqlValue)>,
    pub parameters: Vec<(String, Value)>,
    pub subquery: Option<String>, // For WHERE in (SELECT...) clauses
    pub parallel: bool,           // For PARALLEL flag
}

impl Relation {
    pub fn new(direction: RelationDirection, edge: &str, target: &str) -> Self {
        Self {
            direction,
            edge: edge.to_string(),
            target: target.to_string(),
            recursive: None,
            alias: None,
            return_fields: Vec::new(),
            conditions: Vec::new(),
            subquery: None,
            parameters: Vec::new(),
            parallel: false,
        }
    }
}

impl Display for Relation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut rel_str = String::new();

        // Add recursive depth if present
        if let Some(depth) = &self.recursive {
            rel_str.push_str(" @.");
            match depth {
                RecursiveDepth::Fixed(n) => rel_str.push_str(&format!("{{{}}}", n)),
                RecursiveDepth::Range(min, max) => {
                    rel_str.push_str(&format!("{{{}..{}}}", min, max))
                }
                RecursiveDepth::OpenEnded(max) => {
                    if let Some(max) = max {
                        rel_str.push_str(&format!("{{..{}}}", max));
                    } else {
                        rel_str.push_str("{..}");
                    }
                }
            }

            // For recursive queries, if we have return fields, use the field collection
            // syntax
            if !self.return_fields.is_empty() {
                rel_str.push_str(".{");
                let fields = self
                    .return_fields
                    .iter()
                    .map(|f| {
                        if f.contains(" AS ") {
                            // Convert "field AS alias" to "alias: field"
                            let parts: Vec<&str> = f.split(" AS ").collect();
                            format!("{}: {}", parts[1], parts[0])
                        } else {
                            f.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                rel_str.push_str(&fields);
                rel_str.push('}');
                return write!(f, "{}", rel_str); // Early return as we don't need edge/target for
                                // field collection
            }
        }

        rel_str.push_str(self.direction.to_string().as_str());

        // Handle conditions and subqueries
        if !self.conditions.is_empty() || self.subquery.is_some() {
            rel_str.push('(');
            rel_str.push_str(&self.edge);

            let mut clauses = Vec::new();

            // Add regular conditions
            if !self.conditions.is_empty() {
                let conditions: Vec<String> = self
                    .conditions
                    .iter()
                    .enumerate()
                    .map(|(_i, (field, op, value))| {
                        format!("{} {} {}", field, String::from(op.clone()), value)
                    })
                    .collect();
                clauses.push(format!("WHERE {}", conditions.join(" AND ")));
            }

            // Add subquery if present
            if let Some(subquery) = &self.subquery {
                let clean_subquery = subquery.trim_end_matches(';');
                clauses.push(format!("WHERE in ({})", clean_subquery));
            }

            rel_str.push_str(&format!(" {}", clauses.join(" ")));
            rel_str.push(')');
        }
        // Handle return fields
        else if !self.return_fields.is_empty() {
            rel_str.push_str(&self.edge);
            rel_str.push_str(&format!("[{}]", self.return_fields.join(", ")));
        }
        // Simple case
        else {
            rel_str.push_str(&self.edge);
        }

        // Add target
        rel_str.push_str(&format!("{}{}", self.direction, self.target));

        // Add PARALLEL flag if set
        if self.parallel {
            rel_str.push_str(" PARALLEL");
        }

        if let Some(alias) = &self.alias {
            rel_str.push_str(&format!(" AS {}", alias));
        }
        write!(f, "{}", rel_str)
    }
}

/// Trait for graph traversal operations
pub trait GraphTraversal {
    /// Add a graph traversal step
    fn traverse(self, rel_type: RelationDirection, edge: &str, target: &str) -> Self;

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
    fn traverse(mut self, rel_type: RelationDirection, edge: &str, target: &str) -> Self {
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
