use crate::{FieldType, Permission, SurrealDB};
use anyhow::{anyhow, bail};
use std::fmt::Display;
use tracing::{error, info};

#[derive(Clone, Debug, Default)]
pub struct DefineFieldStatement {
    pub(crate) name: Option<String>,
    pub(crate) table_name: Option<String>,
    pub(crate) column_type: Option<FieldType>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) null: bool,
    pub(crate) default: Option<String>,
    pub(crate) assert: Option<String>,
    pub(crate) permissions: Option<Vec<Permission>>,
    pub(crate) value: Option<String>,
    pub(crate) readonly: bool,
    pub(crate) flexible: bool,
    pub(crate) comment: Option<String>,
}

impl DefineFieldStatement {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn table_name(mut self, table_name: impl Into<String>) -> Self {
        self.table_name = Some(table_name.into());
        self
    }

    pub fn column_type(mut self, column_type: impl Into<FieldType>) -> Self {
        self.column_type = Some(column_type.into());
        self
    }

    pub fn null(mut self) -> Self {
        self.null = true;
        self
    }

    pub fn default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    pub fn assert(mut self, assert: impl Into<String>) -> Self {
        self.assert = Some(assert.into());
        self
    }

    pub fn permissions(mut self, permissions: impl Into<Vec<Permission>>) -> Self {
        self.permissions = Some(permissions.into());
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    pub fn flexible(mut self) -> Self {
        self.flexible = true;
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

    pub fn build(&self) -> crate::Result<String> {
        let mut stmt = String::new();
        stmt.push_str("DEFINE FIELD ");
        if let Some(name) = &self.name {
            if name == "id" {
                return Ok("".to_string());
            }
            stmt.push_str(name.as_str());
        } else {
            bail!("Field name is required");
        }

        if self.if_not_exists {
            stmt.push_str(" IF NOT EXISTS");
        }else if self.overwrite {
            stmt.push_str(" OVERWRITE");
        }

        if let Some(table_name) = &self.table_name {
            stmt.push_str(" ON TABLE ");
            stmt.push_str(table_name.as_str());
        } else {
            bail!("Table name is required")
        }

        if self.flexible {
            stmt.push_str(" FLEXIBLE");
        }

        if let Some(column_type) = &self.column_type {
            stmt.push_str(&format!(" TYPE {}", column_type));
        }

        if self.null {
            stmt.push_str("|null ");
        }

        if let Some(default) = &self.default {
            stmt.push_str(&format!(" DEFAULT {}", default));
        }

        if let Some(assert) = &self.assert {
            stmt.push_str(&format!(" ASSERT {}", assert));
        }

        if let Some(value) = &self.value {
            stmt.push_str(&format!(" VALUE {}", value));
        }

        if let Some(permissions) = &self.permissions {
            if !permissions.is_empty() {
                stmt.push_str(" PERMISSIONS ");
                for perms in permissions {
                    stmt.push_str(format!("{}", perms).as_str());
                }
            }
        }

        if self.readonly {
            stmt.push_str(" READONLY");
        }

        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
        }

        stmt.push(';');
        Ok(stmt)
    }

    pub async fn execute(self, conn: SurrealDB) -> anyhow::Result<Vec<serde_json::Value>> {
        let query = self.build()?;
        info!("Executing query: {}", query);

        let mut surreal_query = conn.query(query);

        let res = surreal_query.await?.take(0);
        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                error!("Query execution failed: {:?}", e);
                Err(anyhow!(e))
            }
        }
    }
}

impl Display for DefineFieldStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap())
    }
}
