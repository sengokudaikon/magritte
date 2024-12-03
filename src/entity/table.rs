use crate::prelude::{ColumnTrait, EventTrait, IndexTrait, RelationTrait};
use magritte_query::types::{Permission, RecordRef, RecordType, SchemaType, SurrealId, TableType};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::str::FromStr;
use std::time::Duration;
use serde::de::DeserializeOwned;
use surrealdb::{sql, RecordId};
use surrealdb::sql::Thing;
use magritte_query::conditions::Operator;
use magritte_query::delete::DeleteStatement;
use magritte_query::Query;
use magritte_query::select::SelectStatement;
use magritte_query::update::UpdateStatement;
use magritte_query::upsert::UpsertStatement;
use magritte_query::wheres::WhereClause;
/// An abstract base class for defining Entities.
///
/// This trait provides an API for you to inspect its properties
/// - Column (implemented [`ColumnTrait`])
/// - Relation (implemented [`RelationTrait`])
/// - Index (implemented [`IndexTrait`])
/// - Event (implemented [`EventTrait`])
///
/// This trait also provides an API for CRUD actions
/// - Select: `find`, `find_*`
/// - Insert: `insert`, `insert_*`
/// - Update: `update`, `update_*`
/// - Delete: `delete`, `delete_*`
pub trait TableTrait: TableType {
    fn def(&self) -> TableDef;

    fn to_statement(&self) -> String {
        self.def().to_statement()
    }

    fn as_record(&self) -> RecordRef<Self>
    where
        Self: Sized,
    {
        RecordRef::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableDef {
    pub(crate) name: String,
    pub(crate) schema_type: SchemaType,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) permissions: Option<Vec<Permission>>,
    pub(crate) drop: bool,
    pub(crate) as_select: Option<String>,
    pub(crate) changefeed: Option<(Duration, bool)>,
    pub(crate) comment: Option<String>,
}

impl TableDef {
    pub fn new(
        name: impl Into<String>,
        schema_type: impl Into<String>,
        overwrite: bool,
        if_not_exists: bool,
        permissions: Option<Vec<String>>,
        drop: bool,
        as_select: Option<String>,
        changefeed: Option<(u64, bool)>,
        comment: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            schema_type: SchemaType::from(schema_type.into()),
            overwrite,
            if_not_exists,
            permissions: permissions.as_ref().map(|pers|pers.iter().map(|p| Permission::from(p)).collect()),
            drop,
            as_select,
            changefeed: changefeed.map(|c| (Duration::from_mins(c.0), c.1)),
            comment,
        }
    }
    pub fn table_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn schema_type(&self) -> &SchemaType {
        &self.schema_type
    }
    pub fn is_overwrite(&self) -> bool {
        self.overwrite
    }

    pub fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    pub fn permissions(&self) -> &[Permission] {
        self.permissions.as_deref().unwrap_or(&[])
    }

    pub fn is_drop(&self) -> bool {
        self.drop
    }

    pub fn as_select(&self) -> Option<&str>{
        self.as_select.as_ref().map(|s| s.as_str())
    }

    pub fn changefeed(&self) -> Option<(Duration, bool)> {
        self.changefeed
    }

    pub fn comment(&self) -> Option<&str> {
        self.comment.as_ref().map(|s| s.as_str())
    }

    pub fn to_statement(&self) -> String {
        let mut stmt = format!("DEFINE TABLE {} TYPE NORMAL", self.name);

        match self.schema_type {
            SchemaType::Schemafull => stmt.push_str(" SCHEMAFULL "),
            SchemaType::Schemaless => stmt.push_str(" SCHEMALESS "),
        }

        if self.drop {
            stmt.push_str("DROP ");
        }
        if self.overwrite {
            stmt.push_str("OVERWRITE ");
        } else if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        }
        if let Some(as_select) = &self.as_select {
            stmt.push_str(format!(" AS SELECT {}", as_select).as_str());
        }
        if let Some(changefeed) = &self.changefeed {
            stmt.push_str(format!( "CHANGEFEED {}", changefeed.0.as_secs()).as_str());
            if changefeed.1 {
                stmt.push_str(" INCLUDE ORIGINAL ");
            }
        }

        if let Some(permissions) = &self.permissions {
            if !permissions.is_empty() {
                stmt.push_str(" PERMISSIONS ");
                for perms in permissions {
                    stmt.push_str(format!("{}", perms).as_str());
                }
            }
        }

        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
        }
        stmt.push(';');
        stmt
    }
}