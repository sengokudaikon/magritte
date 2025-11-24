use crate::entity::{HasColumns, HasEvents, HasIndexes, HasRelations};
use crate::{ColumnTrait, EventTrait, IndexTrait, RelationTrait};
use anyhow::anyhow;
use magritte_core::{Permission, RecordRef, Relations, SchemaType, TableType};
use magritte_query::define::define_table::DefineTableStatement;
use magritte_query::Define;
use std::fmt::{Debug, Display};
use std::time::Duration;

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
pub trait TableTrait: TableType + HasColumns {
    fn def() -> TableDef;
    fn def_owned(&self) -> TableDef {
        Self::def()
    }

    fn to_statement() -> DefineTableStatement<Self> {
        Self::def().to_statement()
    }

    fn to_statement_owned(&self) -> DefineTableStatement<Self> {
        Self::def_owned(self).to_statement()
    }

    fn columns() -> Vec<impl ColumnTrait>
    where
        Self: Sized,
    {
        <Self as HasColumns>::columns()
    }

    fn indexes() -> Vec<impl IndexTrait>
    where
        Self: Sized,
        Self: HasIndexes,
    {
        <Self as HasIndexes>::indexes()
    }

    fn events() -> Vec<impl EventTrait>
    where
        Self: Sized,
        Self: HasEvents,
    {
        <Self as HasEvents>::events()
    }

    fn relations() -> Vec<impl Relations>
    where
        Self: Sized,
        Self: HasRelations,
    {
        <Self as HasRelations>::relations()
    }

    fn as_record(&self) -> RecordRef<Self> {
        RecordRef::new()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
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
    #[allow(clippy::too_many_arguments)]
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
            permissions: permissions
                .as_ref()
                .map(|pers| pers.iter().map(Permission::from).collect()),
            drop,
            as_select,
            changefeed: changefeed.map(|c| {
                let seconds =
                    c.0.checked_mul(60)
                        .expect("changefeed duration overflow when converting minutes to seconds");
                (Duration::from_secs(seconds), c.1)
            }),
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

    pub fn as_select(&self) -> Option<&str> {
        self.as_select.as_deref()
    }

    pub fn changefeed(&self) -> Option<(Duration, bool)> {
        self.changefeed
    }

    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }

    pub fn to_statement<T: TableTrait>(&self) -> DefineTableStatement<T> {
        let mut def = Define::table()
            .name(self.name.clone())
            .schema_type(self.schema_type.clone());

        if self.overwrite {
            def = def.overwrite();
        } else if self.if_not_exists {
            def = def.if_not_exists();
        } else if self.drop {
            def = def.drop();
        }
        if let Some(as_select) = &self.as_select {
            def = def.as_select(as_select.as_str().parse().unwrap());
        }
        if let Some(changefeed) = &self.changefeed {
            def = def.changefeed(changefeed.0, changefeed.1);
        }

        if let Some(permissions) = &self.permissions {
            def = def.permissions(permissions.clone());
        }

        if let Some(comment) = &self.comment {
            def = def.comment(comment.clone());
        }

        def
    }
}

impl<T> From<DefineTableStatement<T>> for TableDef
where
    T: TableTrait,
{
    fn from(value: DefineTableStatement<T>) -> Self {
        TableDef::new(
            value
                .get_name()
                .ok_or_else(|| anyhow!("Table name is required"))
                .unwrap(),
            value.get_schema_type().unwrap().to_string(),
            value.get_overwrite(),
            value.get_if_not_exists(),
            value
                .get_permissions()
                .map(|p| p.iter().map(|p| p.to_string()).collect()),
            value.get_drop(),
            value.get_as_select().map(|f| f.to_string()),
            value.get_changefeed().map(|(d, t)| (d.as_secs(), t)),
            value.get_comment(),
        )
    }
}
