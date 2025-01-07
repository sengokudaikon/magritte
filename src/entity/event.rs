use anyhow::bail;
use magritte_query::types::{EventType, TableType};
use magritte_query::{DefineEventStatement, NamedType, RecordType};
use std::fmt::{Debug, Display};

/// Defines an Event for a Table
#[derive(Debug, Clone, PartialEq)]
pub struct EventDef {
    pub name: String,
    pub(crate) table: String,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) when: String,
    pub(crate) then: String,
    pub(crate) comment: Option<String>,
}

pub trait EventTrait: EventType {
    type EntityName: NamedType;

    fn def(&self) -> EventDef;
    fn to_statement(&self) -> anyhow::Result<DefineEventStatement> {
        self.def().to_statement()
    }
}

impl EventDef {
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
            when: when.into(),
            then: then.into(),
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
        self.comment.as_deref()
    }
    pub fn is_overwrite(&self) -> bool {
        self.overwrite
    }
    pub fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }
    pub fn to_statement(&self) -> anyhow::Result<DefineEventStatement> {
        let mut def = DefineEventStatement::new();
        if self.name == "".to_string() {
            return Ok(def);
        }

        def = def.name(&self.name).table(&self.table);

        if self.overwrite {
            def = def.overwrite();
        } else if self.if_not_exists {
            def = def.if_not_exists();
        }

        if let Some(comment) = &self.comment {
            def = def.comment(comment);
        }

        def = def.when(&self.when);
        def = def.then(&self.then);

        Ok(def)
    }
}
