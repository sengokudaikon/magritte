use magritte_query::types::{RelationType, TableType};
use std::fmt::{Debug, Display};

/// Defines a Relation between Tables through an Edge
#[derive(Debug, Clone, PartialEq)]
pub struct RelationDef {
    pub(crate) from: String,    // Source record
    pub(crate) to: String,      // Target record
    pub(crate) via: String,     // Edge Table name
    pub(crate) content: Option<String>, // Optional content for the edge
}

pub trait RelationTrait: RelationType {
    type EntityName: TableType;  // The Table that owns this relation

    fn def(&self) -> RelationDef;

    fn to_relate_statement(&self) -> String {
        let def = self.def();
        let mut stmt = format!("RELATE {}->{}->{}", def.from, def.via, def.to);
        
        if let Some(content) = def.content {
            stmt.push_str(&format!(" CONTENT {}", content));
        }

        stmt.push(';');
        stmt
    }
}

impl RelationDef {
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        via: impl Into<String>,
        content: impl Into<Option<String>>
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            via: via.into(),
            content: content.into(),
        }
    }

    pub fn relation_name(&self) -> &str {
        self.via.as_str()
    }
    pub fn relation_from(&self) -> &str {
        self.from.as_str()
    }
    pub fn relation_to(&self) -> &str {
        self.to.as_str()
    }
    pub fn content(&self) -> Option<&str> {
        self.content.as_ref().map(|c| c.as_str())
    }
}
