use crate::database::{QueryType, SurrealDB};
use crate::IndexSpecifics;
use anyhow::bail;
use std::fmt::Display;

#[derive(Default, Debug, Clone)]
pub struct DefineIndexStatement {
    pub(crate) name: Option<String>,
    pub(crate) table: Option<String>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) fields: Option<Vec<String>>,
    pub(crate) columns: Option<Vec<String>>,
    pub(crate) unique: bool,
    pub(crate) specifics: IndexSpecifics,
    pub(crate) comment: Option<String>,
    pub(crate) concurrently: bool,
}

impl DefineIndexStatement {
    pub fn new() -> Self {
        Self {
            table: None,
            name: None,
            if_not_exists: false,
            fields: None,
            columns: None,
            unique: false,
            specifics: Default::default(),
            comment: None,
            overwrite: false,
            concurrently: false,
        }
    }

    pub fn table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn fields(mut self, fields: impl Into<Vec<String>>) -> Self {
        self.fields = Some(fields.into());
        self
    }

    pub fn columns(mut self, columns: impl Into<Vec<String>>) -> Self {
        self.columns = Some(columns.into());
        self
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn overwrite(mut self) -> Self {
        self.overwrite = true;
        self
    }

    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    pub fn specifics(mut self, specifics: impl Into<IndexSpecifics>) -> Self {
        self.specifics = specifics.into();
        self
    }

    pub fn concurrently(mut self) -> Self {
        self.concurrently = true;
        self
    }

    pub fn build(&self) -> anyhow::Result<String> {
        let mut stmt = String::new();
        if self.name.is_none() {
            return Ok(stmt);
        }
        stmt.push_str("DEFINE INDEX ");
        if self.overwrite {
            stmt.push_str("OVERWRITE ");
        } else if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        }
        if let Some(name) = &self.name {
            stmt.push_str(name.as_str());
        } else {
            bail!("Index name is required");
        }

        stmt.push_str(" ON ");

        if let Some(table) = &self.table {
            stmt.push_str(table.as_str());
        } else {
            bail!("Table name is required");
        }

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

    pub async fn execute(self, conn: &SurrealDB) -> anyhow::Result<Vec<serde_json::Value>> {
        conn.execute(self.build()?, vec![], QueryType::Schema, None).await
    }
}

impl Display for DefineIndexStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap())
    }
}
