use magritte_query::types::IndexType;
use magritte_query::{Define, DefineIndexStatement, IndexSpecifics, NamedType};
use std::fmt::{Debug, Display};

/// Defines an Index for a Table
#[derive(Debug, Clone, PartialEq)]
pub struct IndexDef {
    pub(crate) name: String,
    pub(crate) table: String,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) fields: Option<Vec<String>>,
    pub(crate) columns: Option<Vec<String>>,
    pub(crate) unique: bool,
    pub(crate) specifics: IndexSpecifics,
    pub(crate) comment: Option<String>,
    pub(crate) concurrently: bool,
}

pub trait IndexTrait: IndexType {
    type EntityName: NamedType;

    fn def(&self) -> IndexDef;

    fn to_statement(&self) -> DefineIndexStatement {
        self.def().to_statement()
    }
}

impl IndexDef {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        table: impl Into<String>,
        fields: Option<Vec<String>>,
        columns: Option<Vec<String>>,
        overwrite: bool,
        if_not_exists: bool,
        unique: bool,
        specifics: String,
        comment: Option<String>,
        concurrently: bool,
    ) -> Self {
        Self {
            name: name.into(),
            table: table.into(),
            overwrite,
            if_not_exists,
            fields,
            columns,
            unique,
            specifics: IndexSpecifics::from(&specifics),
            comment,
            concurrently,
        }
    }
    pub fn index_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn table_name(&self) -> &str {
        self.table.as_str()
    }
    pub fn is_overwrite(&self) -> bool {
        self.overwrite
    }

    pub fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }
    pub fn is_concurrent(&self) -> bool {
        self.concurrently
    }
    pub fn is_unique(&self) -> bool {
        self.unique
    }
    pub fn fields(&self) -> Option<Vec<&str>> {
        self.fields
            .as_ref()
            .map(|fields| fields.iter().map(|f| f.as_str()).collect())
    }
    pub fn columns(&self) -> Option<Vec<&str>> {
        self.columns
            .as_ref()
            .map(|columns| columns.iter().map(|c| c.as_str()).collect())
    }
    pub fn specifics(&self) -> &IndexSpecifics {
        &self.specifics
    }
    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
    pub fn to_statement(&self) -> DefineIndexStatement {
        let mut def = Define::index()
            .name(self.name.clone())
            .table(self.table.clone());

        if self.overwrite {
            def = def.overwrite();
        } else if self.if_not_exists {
            def = def.if_not_exists();
        }

        if self.unique {
            def = def.unique();
        }

        if self.concurrently {
            def = def.concurrently();
        }

        def = def.specifics(self.specifics.clone());

        if let Some(fields) = &self.fields {
            def = def.fields(fields.clone());
        }
        if let Some(columns) = &self.columns {
            def = def.columns(columns.clone());
        }
        if let Some(comment) = &self.comment {
            def = def.comment(comment.clone());
        }
        def
    }
}
