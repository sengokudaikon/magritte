use crate::graph::Relation;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Projection {
    All,
    Field(String),
    FieldAs(String, String),
    Fields(Vec<String>),
    FieldsAs(Vec<(String, String)>),
    Raw(String),
    RawAs(String, String),
    Subquery(String, Option<String>),
    RelationWildcardAs(String),
    RelationInverseWildcardAs(String),
    RelationBidirectionalWildcardAs(String),
    Relation(Relation),
}

impl Display for Projection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Projection::All => write!(f, "*"),
            Projection::Field(field) => write!(f, "{}", field),
            Projection::FieldAs(field, alias) => write!(f, "{} AS {}", field, alias),
            Projection::Fields(fields) => write!(f, "{}", fields.join(", ")),
            Projection::FieldsAs(fields) => write!(
                f,
                "{}",
                fields
                    .iter()
                    .map(|(field, alias)| format!("{} AS {}", field, alias))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Projection::Raw(raw) => write!(f, "{}", raw),
            Projection::RawAs(raw, alias) => write!(f, "{} AS {}", raw, alias),
            Projection::Subquery(query, alias) => {
                if let Some(alias) = alias {
                    write!(f, "({}) AS {}", query, alias)
                } else {
                    write!(f, "({})", query)
                }
            }
            Projection::RelationWildcardAs(alias) => write!(f, "->?->? AS {}", alias),
            Projection::RelationInverseWildcardAs(alias) => write!(f, "<-?<-? AS {}", alias),
            Projection::RelationBidirectionalWildcardAs(alias) => {
                write!(f, "<->?<->? AS {}", alias)
            }
            Projection::Relation(relation) => write!(f, "{}", relation),
        }
    }
}
