use magritte_query::types::{ColumnType, NamedType, Permission};
use magritte_query::{Define, DefineFieldStatement};
use std::fmt::Display;
use crate::TableTrait;

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
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) comment: Option<String>,
}

pub trait ColumnTrait: ColumnType {
    type EntityName: NamedType;

    fn def(&self) -> ColumnDef;
    fn to_statement(&self) -> DefineFieldStatement {
        self.def().to_statement()
    }
}

impl ColumnDef {
    #[allow(clippy::too_many_arguments)]
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
        overwrite: bool,
        if_not_exists: bool,
        comment: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            table_name: table_name.into(),
            col_type: col_type.into(),
            null,
            default,
            assert,
            permissions: permissions
                .as_ref()
                .map(|pers| pers.iter().map(Permission::from).collect()),
            value,
            readonly,
            flexible,
            overwrite,
            if_not_exists,
            comment,
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

    pub fn is_record(&self) -> bool {
        self.col_type.contains("record")
    }

    pub fn as_record<T: TableTrait>(&self) -> Option<T> {
        if self.is_record() {
            let temp = self.value.clone()?;
            serde_json::from_str(&temp).ok()
        } else {
            None
        }
    }

    pub fn is_readonly(&self) -> bool {
        self.readonly
    }

    pub fn is_flexible(&self) -> bool {
        self.flexible
    }

    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }

    pub fn permissions(&self) -> &[Permission] {
        self.permissions.as_deref().unwrap_or(&[])
    }

    pub fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }

    pub fn assert(&self) -> Option<&str> {
        self.assert.as_deref()
    }

    pub fn to_statement(&self) -> DefineFieldStatement {
        let mut def = Define::field();
        def = def.name(self.name.clone());
        def = def.table_name(self.table_name.clone());
        def = def.column_type(self.col_type.clone());

        if self.null {
            def = def.null();
        }
        if let Some(default) = self.default.clone() {
            def = def.default(default);
        }

        if let Some(assert) = self.assert.clone() {
            def = def.assert(assert);
        }

        if let Some(permissions) = self.permissions.clone() {
            def = def.permissions(permissions);
        }

        if let Some(value) = self.value.clone() {
            def = def.value(value);
        }

        if self.readonly {
            def = def.readonly();
        }

        if self.flexible {
            def = def.flexible();
        }
        if let Some(comment) = self.comment.clone() {
            def = def.comment(comment);
        }
        def
    }
}
