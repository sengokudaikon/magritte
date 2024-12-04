use magritte_query::types::{EventType, TableType};
use std::fmt::{Debug, Display};

/// Defines an Event for a Table
#[derive(Debug, Clone, PartialEq)]
pub struct EventDef {
    pub(crate) name: String,
    pub(crate) table: String,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) when: String,
    pub(crate) then: String,
    pub(crate) comment: Option<String>,
}

pub trait EventTrait: EventType {
    type EntityName: TableType;

    fn def(&self) -> EventDef;
    fn to_statement(&self) -> String {
        self.def().to_statement()
    }
}

impl EventDef {
    fn cleanup(s: impl Into<String>) -> String {
        let mut s = s.into().replace("\"","");
        s = s.replace("var:", "$");
        s
    }
    pub fn new(
        name: impl Into<String>,
        table: impl Into<String>,
        when: impl Into<String>,
        then: impl Into<String>,
        comment: Option<String>,
        overwrite: bool,
        if_not_exists: bool,
    ) -> Self {
        Self {
            name: name.into(),
            table: table.into(),
            overwrite,
            if_not_exists,
            when: Self::cleanup(when),
            then: Self::cleanup(then),
            comment,
        }
    }
    pub fn event_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn table_name(&self) -> &str {
        self.table.as_str()
    }
    pub fn event_when(&self) -> &str {
        self.when.as_str()
    }
    pub fn event_then(&self) -> &str {
        self.then.as_str()
    }
    pub fn comment(&self) -> Option<&str> {
        self.comment.as_ref().map(|c| c.as_str())
    }
    pub fn is_overwrite(&self) -> bool {
        self.overwrite
    }
    pub fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }
    pub fn to_statement(&self) -> String {
        let mut stmt = String::new();
        stmt.push_str("DEFINE EVENT ");
        if self.overwrite {
            stmt.push_str("OVERWRITE ");
        } else if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        }
        stmt.push_str(&self.name);

        stmt.push_str(" ON TABLE ");
        stmt.push_str(&self.table);

        stmt.push_str(" WHEN ");
        stmt.push_str(&*Self::cleanup(&self.when));

        stmt.push_str(" THEN ");
        stmt.push_str(&*Self::cleanup(&self.then));

        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
        }

        stmt.push(';');
        stmt
    }
}
