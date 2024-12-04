use anyhow::bail;
use anyhow::Result;
use std::fmt::{Debug, Display};
use std::str::FromStr;
use crate::prelude::IndexSpecifics;
use magritte_query::types::{IndexType, TableType};

/// Defines an Index for a Table
#[derive(Debug, Clone, PartialEq)]
pub struct IndexDef {
    pub(crate) name: String,
    pub(crate) table: String,
    pub(crate) overwrite: bool,
    pub(crate) use_table: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) fields: Option<Vec<String>>,
    pub(crate) columns: Option<Vec<String>>,
    pub(crate) unique: bool,
    pub(crate) specifics: IndexSpecifics,
    pub(crate) comment: Option<String>,
    pub(crate) concurrently: bool,
}

pub trait IndexTrait: IndexType {
    type EntityName: TableType;

    fn def(&self) -> IndexDef;

    fn to_statement(&self) -> Result<String> {
        self.def().to_statement()
    }
}

impl IndexDef {
    pub fn new(
        name: impl Into<String>,
        table: impl Into<String>,
        fields: Option<Vec<String>>,
        columns: Option<Vec<String>>,
        overwrite: bool,
        use_table: bool,
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
            use_table,
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
        self.comment.as_ref().map(|c| c.as_str())
    }
    pub fn to_statement(&self) -> anyhow::Result<String> {
        let mut stmt = String::new();
        stmt.push_str("DEFINE INDEX ");
        if self.overwrite {
            stmt.push_str("OVERWRITE ");
        } else if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        }
        stmt.push_str(&*self.name);

        stmt.push_str(" ON ");
        if self.use_table {
            stmt.push_str("TABLE ");
        }
        stmt.push_str(&*self.table);

        if let Some(fields) = &self.fields {
            stmt.push_str(" FIELDS ");
            if fields.len() == 1 {
                stmt.push_str(fields.first().unwrap().as_str());
            } else if fields.len() > 1 {
                stmt.push_str(fields.join(", ").as_str());
            }
        } else if let Some(columns) = &self.columns {
            stmt.push_str(" COLUMNS ");
            if columns.len() == 1 {
                stmt.push_str(columns.first().unwrap().as_str());
            } else if columns.len() > 1 {
                stmt.push_str(columns.join(", ").as_str());
            }
        } else {
            bail!("No fields or columns provided")
        }

        stmt.push_str(self.specifics.to_string().as_str());

        if self.unique {
            stmt.push_str(" UNIQUE");
        }

        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
        }

        if self.concurrently {
            stmt.push_str(" CONCURRENTLY");
        }

        stmt.push(';');
        Ok(stmt)
    }
}
