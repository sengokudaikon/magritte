use magritte_query::types::{ColumnType, EdgeType, Permission, SchemaType, TableType};
use std::fmt::{Debug, Display};

/// Defines an Edge for a Table
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeDef {
    pub(crate) name: String,
    pub(crate) schema_type: SchemaType,
    pub(crate) permissions: Option<Vec<Permission>>,
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) drop: bool,
    pub(crate) enforced: bool,
}

pub trait EdgeTrait: EdgeType {
    type EntityFrom: TableType;
    type EntityTo: TableType;
    fn def(&self) -> EdgeDef;
    fn to_statement(&self) -> String {
        self.def().to_statement()
    }
}

impl EdgeDef {
    pub fn new(
        name: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        schema_type: impl Into<String>,
        permissions: Option<Vec<String>>,
        overwrite: bool,
        if_not_exists: bool,
        drop: bool,
        enforced: bool,
    ) -> Self {
        Self {
            name: name.into(),
            schema_type: SchemaType::from(schema_type.into()),
            permissions: permissions.as_ref().map(|pers|pers.iter().map(|c| Permission::from(c)).collect()),
            from: from.into(),
            to: to.into(),
            overwrite,
            if_not_exists,
            drop,
            enforced,
        }
    }
    pub fn edge_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn schema_type(&self) -> &SchemaType {
        &self.schema_type
    }
    pub fn permissions(&self) -> &[Permission] {
        self.permissions.as_deref().unwrap_or(&[])
    }
    pub fn edge_from(&self) -> &str {
        self.from.as_str()
    }
    pub fn edge_to(&self) -> &str {
        self.to.as_str()
    }
    pub fn is_enforced(&self) -> bool {
        self.enforced
    }
    pub fn is_overwrite(&self) -> bool {
        self.overwrite
    }
    pub fn is_drop(&self) -> bool {
        self.drop
    }
    pub fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }
    pub fn to_statement(&self) -> String {
        let mut stmt = String::from("DEFINE TABLE");

        if self.overwrite {
            stmt.push_str(" OVERWRITE");
        } else if self.if_not_exists {
            stmt.push_str(" IF NOT EXISTS");
        } else if self.drop {
            stmt.push_str(" DROP");
        }

        stmt.push_str(&format!(
            " {} {} TYPE RELATION FROM {} TO {}",
            self.name, self.schema_type, self.from, self.to
        ));

        if self.enforced {
            stmt.push_str(" ENFORCED");
        }

        if let Some(permissions) = &self.permissions {
            if !permissions.is_empty() {
                stmt.push_str(" PERMISSIONS ");
                for perms in permissions {
                    stmt.push_str(format!("{}", perms).as_str());
                }
            }
        }

        stmt.push(';');
        stmt
    }
}
