use crate::{CanCallFunctions, HasVectorConditions, VectorFunction};
use magritte_core::{Indexable, VectorCondition, VectorDistance};

/// Extension trait for vector search functionality
pub trait VectorSearchable {
    /// Perform nearest neighbor search using vector similarity
    fn vector_nearest(
        self,
        field: &str,
        vector: Vec<f32>,
        k: usize,
        operator: VectorDistance,
    ) -> Self;

    /// Perform similarity search with optional threshold
    fn vector_similarity(
        self,
        field: &str,
        vector: Vec<f32>,
        distance: VectorDistance,
        threshold: Option<f32>,
    ) -> Self;

    /// Perform batch nearest neighbor search
    fn vector_batch_nearest(
        self,
        conditions: Vec<(String, Vec<f32>, VectorDistance, usize)>,
    ) -> Self;

    /// Perform batch similarity search
    fn vector_batch_similarity(
        self,
        conditions: Vec<(String, Vec<f32>, VectorDistance, Option<f32>)>,
    ) -> Self;

    /// Calculate vector similarity score in SELECT
    fn vector_similarity_score(
        self,
        field: &str,
        vector: Vec<f32>,
        distance: VectorDistance,
    ) -> Self;

    /// Get KNN distance in SELECT (requires prior KNN search)
    fn vector_knn_distance(self) -> Self;

    /// Define vector index hint
    fn with_vector_index(self, index_name: &str) -> Self;

    fn build_vector_conditions(conditions: &[VectorCondition]) -> String {
        let mut query = String::new();

        for condition in conditions {
            match condition {
                VectorCondition::Similarity {
                    field,
                    vector,
                    distance: operator,
                    threshold,
                } => {
                    query.push_str(&format!(
                        " WHERE vector::similarity::{}({}, {}) ",
                        operator.to_string().to_lowercase(),
                        field,
                        vector
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    ));
                    if let Some(t) = threshold {
                        query.push_str(&format!(">= {}", t));
                    }
                }
                VectorCondition::Nearest {
                    field,
                    vector,
                    k,
                    distance: operator,
                } => {
                    query.push_str(&format!(
                        " ORDER BY vector::similarity::{}({}, {}) DESC LIMIT {}",
                        operator.to_string().to_lowercase(),
                        field,
                        vector
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .join(","),
                        k
                    ));
                }
                VectorCondition::BatchSimilarity { conditions } => {
                    for (field, vector, operator, threshold) in conditions {
                        query.push_str(&format!(
                            " WHERE vector::similarity::{}({}, {}) ",
                            operator.to_string().to_lowercase(),
                            field,
                            vector
                                .iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<_>>()
                                .join(",")
                        ));
                        if let Some(t) = threshold {
                            query.push_str(&format!(">= {}", t));
                        }
                    }
                }
                VectorCondition::BatchNearest { conditions } => {
                    for (field, vector, operator, k) in conditions {
                        query.push_str(&format!(
                            " ORDER BY vector::similarity::{}({}, {}) DESC LIMIT {}",
                            operator.to_string().to_lowercase(),
                            field,
                            vector
                                .iter()
                                .map(|x| x.to_string())
                                .collect::<Vec<_>>()
                                .join(","),
                            k
                        ));
                    }
                }
            }
        }
        query
    }
}

impl<U: HasVectorConditions + CanCallFunctions + Indexable> VectorSearchable for U {
    fn vector_nearest(
        mut self,
        field: &str,
        vector: Vec<f32>,
        k: usize,
        operator: VectorDistance,
    ) -> Self {
        self.get_vector_conditions_mut()
            .push(VectorCondition::Nearest {
                field: field.to_string(),
                vector,
                k,
                distance: operator,
            });
        self
    }

    fn vector_similarity(
        mut self,
        field: &str,
        vector: Vec<f32>,
        operator: VectorDistance,
        threshold: Option<f32>,
    ) -> Self {
        self.get_vector_conditions_mut()
            .push(VectorCondition::Similarity {
                field: field.to_string(),
                vector,
                distance: operator,
                threshold,
            });
        self
    }

    fn vector_batch_nearest(
        mut self,
        conditions: Vec<(String, Vec<f32>, VectorDistance, usize)>,
    ) -> Self {
        self.get_vector_conditions_mut()
            .push(VectorCondition::BatchNearest { conditions });
        self
    }

    fn vector_batch_similarity(
        mut self,
        conditions: Vec<(String, Vec<f32>, VectorDistance, Option<f32>)>,
    ) -> Self {
        self.get_vector_conditions_mut()
            .push(VectorCondition::BatchSimilarity { conditions });
        self
    }

    fn vector_similarity_score(
        self,
        field: &str,
        vector: Vec<f32>,
        operator: VectorDistance,
    ) -> Self {
        match operator {
            VectorDistance::Cosine => self.call_function(VectorFunction::SimilarityCosine(
                field.to_string(),
                format!(
                    "[{}]",
                    vector
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            )),
            VectorDistance::Euclidean => self.call_function(VectorFunction::DistanceEuclidean(
                field.to_string(),
                format!(
                    "[{}]",
                    vector
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            )),
            VectorDistance::Manhattan => self.call_function(VectorFunction::DistanceManhattan(
                field.to_string(),
                format!(
                    "[{}]",
                    vector
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            )),
            VectorDistance::Hamming => self.call_function(VectorFunction::DistanceHamming(
                field.to_string(),
                format!(
                    "[{}]",
                    vector
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            )),
            VectorDistance::Chebyshev => self.call_function(VectorFunction::DistanceChebyshev(
                field.to_string(),
                format!(
                    "[{}]",
                    vector
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            )),
            VectorDistance::Minkowski(p) => self.call_function(VectorFunction::DistanceMinkowski(
                field.to_string(),
                format!(
                    "[{}]",
                    vector
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                p,
            )),
        }
    }

    fn vector_knn_distance(self) -> Self {
        self.call_function(VectorFunction::DistanceKnn)
    }

    fn with_vector_index(mut self, index_name: &str) -> Self {
        if let Some(indexes) = self.with_index_mut() {
            indexes.push(index_name.to_string());
        }
        self
    }
}
