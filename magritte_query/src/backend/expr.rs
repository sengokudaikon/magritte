use crate::backend::value::SqlValue;
use crate::conditions::Operator;
use crate::graph::Relation;
use crate::vector_search::VectorCondition;
use serde_json::Value;

pub trait HasVectorConditions {
    fn get_vector_conditions(&self) -> &Vec<VectorCondition>;
    fn get_vector_conditions_mut(&mut self) -> &mut Vec<VectorCondition>;
}

pub trait HasRelations {
    fn relations(&self) -> &Vec<Relation>;
    fn relations_mut(&mut self) -> &mut Vec<Relation>;
}
pub trait HasParams {
    fn params(&self) -> &Vec<(String, Value)>;
    fn params_mut(&mut self) -> &mut Vec<(String, Value)>;
}
pub trait HasConditions {
    fn conditions_mut(&mut self) -> &mut Vec<(String, Operator, SqlValue)>;
}

pub trait HasLetConditions {
    fn get_lets(&self) -> &Vec<(String, String)>;
    fn get_lets_mut(&mut self) -> &mut Vec<(String, String)>;
}

pub trait Lets {
    fn lets(self, var: &str, expr: &str) -> Self;
}

impl<T: HasLetConditions> Lets for T {
    fn lets(mut self, var: &str, expr: &str) -> Self {
        self.get_lets_mut()
            .push((var.to_string(), expr.to_string()));
        self
    }
}
