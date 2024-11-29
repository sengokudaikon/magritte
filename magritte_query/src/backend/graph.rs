use serde::Serialize;
use serde_json::Value;

use super::conditions::Operator;
use crate::backend::QueryBuilder;
use crate::backend::value::SqlValue;
use crate::expr::HasRelations;
use crate::types::TableType;

#[derive(Clone, Debug, PartialEq)]
pub enum RelationType {
    In,   // <-
    Out,  // ->
    Both, // <->
}

#[derive(Clone, Debug, PartialEq)]
pub enum RecursiveDepth {
    Fixed(usize),             // @.{3}
    Range(usize, usize),      // @.{1..5}
    OpenEnded(Option<usize>), // @.{..} or @.{..256}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Relation {
    pub rel_type: RelationType,
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
    pub fn new(rel_type: RelationType, edge: &str, target: &str) -> Self {
        Self {
            rel_type,
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

    pub fn build_query_part(&self) -> String {
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
                return rel_str; // Early return as we don't need edge/target for
                                // field collection
            }
        }

        let direction = match self.rel_type {
            RelationType::Out => "->",
            RelationType::In => "<-",
            RelationType::Both => "<->",
        };

        rel_str.push_str(direction);

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
        rel_str.push_str(&format!("{}{}", direction, self.target));

        // Add PARALLEL flag if set
        if self.parallel {
            rel_str.push_str(" PARALLEL");
        }

        if let Some(alias) = &self.alias {
            rel_str.push_str(&format!(" AS {}", alias));
        }
        rel_str
    }
}

/// Trait for graph traversal operations
pub trait GraphTraversal {
    /// Add a graph traversal step
    fn traverse(self, rel_type: RelationType, edge: &str, target: &str) -> Self;

    /// Add recursive traversal
    fn recursive(self, depth: RecursiveDepth) -> Self;
    fn with_alias(self, alias: &str) -> Self;

    /// Add return fields from edges [field1, field2]
    fn return_fields(self, fields: Vec<&str>) -> Self;

    /// Add conditions on edges (WHERE clause)
    fn with_edge_condition<V: Serialize>(self, field: &str, op: Operator, value: V) -> Self;

    /// Add subquery in edge conditions
    fn with_edge_subquery<U: TableType, V: QueryBuilder<U>>(self, subquery: V) -> Self;

    /// Enable parallel processing
    fn parallel(self) -> Self;
}

impl<T: HasRelations> GraphTraversal for T {
    fn traverse(mut self, rel_type: RelationType, edge: &str, target: &str) -> Self {
        self.relations_mut()
            .push(Relation::new(rel_type, edge, target));
        self
    }

    fn recursive(mut self, depth: RecursiveDepth) -> Self {
        if let Some(relation) = self.relations_mut().last_mut() {
            relation.recursive = Some(depth);
        }
        self
    }

    fn with_alias(mut self, alias: &str) -> Self {
        if let Some(relation) = self.relations_mut().last_mut() {
            relation.alias = Some(String::from(alias))
        }
        self
    }

    fn return_fields(mut self, fields: Vec<&str>) -> Self {
        if let Some(relation) = self.relations_mut().last_mut() {
            relation.return_fields = fields.into_iter().map(String::from).collect();
        }
        self
    }

    fn with_edge_condition<V: Serialize>(mut self, field: &str, op: Operator, value: V) -> Self {
        let rel_count = &self.relations().len();
        if let Some(relation) = self.relations_mut().last_mut() {
            let param_len = relation.parameters.len();
            let param_name = format!("r{}p{}", rel_count, param_len,);
            let param_value = serde_json::to_value(value).expect("Failed to serialize value");
            let condition = (field.to_string(), op, SqlValue::Param(param_name.clone()));

            relation.parameters.push((param_name, param_value));
            relation.conditions.push(condition);
        }
        self
    }

    fn with_edge_subquery<U: TableType, V: QueryBuilder<U>>(mut self, subquery: V) -> Self {
        if let Some(relation) = self.relations_mut().last_mut() {
            relation.subquery = Some(subquery.build().unwrap_or_default());
        }
        self
    }

    fn parallel(mut self) -> Self {
        if let Some(relation) = self.relations_mut().last_mut() {
            relation.parallel = true;
        }
        self
    }
}
