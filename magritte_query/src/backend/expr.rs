use magritte_core::operator::Operator;
use magritte_core::value::SqlValue;
use magritte_core::{Projection, VectorCondition};
use serde_json::Value;

pub trait HasVectorConditions {
    fn get_vector_conditions(&self) -> &Vec<VectorCondition>;
    fn get_vector_conditions_mut(&mut self) -> &mut Vec<VectorCondition>;
}

pub trait HasProjections {
    fn projections(&self) -> &Vec<Projection>;
    fn projections_mut(&mut self) -> &mut Vec<Projection>;
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

pub trait LetBinding {
    fn let_(self, var: &str, expr: &str) -> Self;
}

impl<T: HasLetConditions> LetBinding for T {
    fn let_(mut self, var: &str, expr: &str) -> Self {
        self.get_lets_mut()
            .push((var.to_string(), expr.to_string()));
        self
    }
}
