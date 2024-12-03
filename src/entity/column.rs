use magritte_query::types::{ColumnType, NamedType, Permission};
use std::fmt::Display;

/// Defines a Column for an Entity
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub(crate) name: String,
    pub(crate) table_name: String,
    pub(crate) col_type: String,
    pub(crate) null: bool,
    pub(crate) default: Option<String>,
    pub(crate) assert: Option<String>,
    pub(crate) permissions: Option<Vec<Permission>>,
    pub(crate) value: Option<String>,
    pub(crate) readonly: bool,
    pub(crate) flexible: bool,
    pub(crate) comment: Option<String>,
}

pub trait ColumnTrait: ColumnType {
    type EntityName: NamedType;

    fn def(&self) -> ColumnDef;
    fn to_statement(&self, table_name: &str) -> String {
        self.def().to_statement()
    }
}

impl ColumnDef {
    pub fn new(
        name: impl Into<String>,
        table_name: impl Into<String>,
        col_type: impl Into<String>,
        default: Option<String>,
        assert: Option<String>,
        permissions: Option<Vec<String>>,
        value: Option<String>,
        null: bool,
        readonly: bool,
        flexible: bool,
        comment: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            table_name: table_name.into(),
            col_type: col_type.into(),
            null,
            default: default.into(),
            assert: assert.into(),
            permissions: permissions
                .as_ref()
                .map(|pers| pers.into_iter().map(|p| Permission::from(p)).collect()),
            value,
            readonly,
            flexible,
            comment: comment.into(),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn table_name(&self) -> &str {
        self.table_name.as_str()
    }

    pub fn column_type(&self) -> &str {
        self.col_type.as_str()
    }

    pub fn is_nullable(&self) -> bool {
        self.null
    }

    pub fn is_readonly(&self) -> bool {
        self.readonly
    }

    pub fn is_flexible(&self) -> bool {
        self.flexible
    }

    pub fn comment(&self) -> Option<&str> {
        self.comment.as_ref().map(|c| c.as_str())
    }

    pub fn permissions(&self) -> &[Permission] {
        self.permissions.as_deref().unwrap_or(&[])
    }

    pub fn default(&self) -> Option<&str> {
        self.default.as_ref().map(|d| d.as_str())
    }

    pub fn assert(&self) -> Option<&str> {
        self.assert.as_ref().map(|a| a.as_str())
    }

    pub fn to_statement(&self) -> String {
        if self.name == "id" {
            return "".to_string();
        }
        let mut stmt = format!("DEFINE FIELD {} ON TABLE {}", self.name, self.table_name);

        if self.flexible {
            stmt.push_str(" FLEXIBLE");
        }
        stmt.push_str(&format!("TYPE {}", self.col_type));

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
        stmt
    }
}
