use anyhow::bail;
use magritte_db::db;
use std::fmt::Display;
use magritte_core::{FieldType, Permission};

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

    pub fn build(&self) -> anyhow::Result<String> {
        let mut stmt = String::new();
        stmt.push_str("DEFINE FIELD ");
        if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        } else if self.overwrite {
            stmt.push_str("OVERWRITE ");
        }

        if let Some(name) = &self.name {
            if name == "id" {
                return Ok("".to_string());
            }
            stmt.push_str(name.as_str());
        } else {
            bail!("Field name is required");
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

    pub async fn execute(self) -> anyhow::Result<Vec<serde_json::Value>> {
        db().execute(self.build()?, vec![]).await
    }
}

impl Display for DefineFieldStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_field() {
        let stmt = DefineFieldStatement::new();
        assert!(stmt.build().is_err());
    }

    #[test]
    fn test_basic_field() {
        let stmt = DefineFieldStatement::new()
            .name("email")
            .table_name("user");

        assert_eq!(stmt.to_string(), "DEFINE FIELD email ON TABLE user;");
    }

    #[test]
    fn test_field_with_type() {
        let stmt = DefineFieldStatement::new()
            .name("email")
            .table_name("user")
            .column_type("string");

        assert_eq!(stmt.to_string(), "DEFINE FIELD email ON TABLE user TYPE string;");
    }

    #[test]
    fn test_field_with_default() {
        let stmt = DefineFieldStatement::new()
            .name("locked")
            .table_name("user")
            .column_type("bool")
            .default("false");

        assert_eq!(stmt.to_string(), "DEFINE FIELD locked ON TABLE user TYPE bool DEFAULT false;");
    }

    #[test]
    fn test_field_with_assert() {
        let stmt = DefineFieldStatement::new()
            .name("email")
            .table_name("user")
            .column_type("string")
            .assert("string::is::email($value)");

        assert_eq!(stmt.to_string(), "DEFINE FIELD email ON TABLE user TYPE string ASSERT string::is::email($value);");
    }

    #[test]
    fn test_field_with_value() {
        let stmt = DefineFieldStatement::new()
            .name("email")
            .table_name("user")
            .column_type("string")
            .value("string::lowercase($value)");

        assert_eq!(stmt.to_string(), "DEFINE FIELD email ON TABLE user TYPE string VALUE string::lowercase($value);");
    }

    #[test]
    fn test_field_with_readonly() {
        let stmt = DefineFieldStatement::new()
            .name("created")
            .table_name("resource")
            .value("time::now()")
            .readonly();

        assert_eq!(stmt.to_string(), "DEFINE FIELD created ON TABLE resource VALUE time::now() READONLY;");
    }

    #[test]
    fn test_field_with_flexible() {
        let stmt = DefineFieldStatement::new()
            .name("metadata")
            .table_name("user")
            .flexible()
            .column_type("object");

        assert_eq!(stmt.to_string(), "DEFINE FIELD metadata ON TABLE user FLEXIBLE TYPE object;");
    }

    #[test]
    fn test_field_with_comment() {
        let stmt = DefineFieldStatement::new()
            .name("email")
            .table_name("user")
            .column_type("string")
            .comment("User's email address");

        assert_eq!(stmt.to_string(), "DEFINE FIELD email ON TABLE user TYPE string COMMENT \"User's email address\";");
    }

    #[test]
    fn test_field_with_overwrite() {
        let stmt = DefineFieldStatement::new()
            .name("email")
            .table_name("user")
            .column_type("string")
            .overwrite();

        assert_eq!(stmt.to_string(), "DEFINE FIELD OVERWRITE email ON TABLE user TYPE string;");
    }

    #[test]
    fn test_field_if_not_exists() {
        let stmt = DefineFieldStatement::new()
            .name("email")
            .table_name("user")
            .column_type("string")
            .if_not_exists();

        assert_eq!(stmt.to_string(), "DEFINE FIELD IF NOT EXISTS email ON TABLE user TYPE string;");
    }

    #[test]
    fn test_field_with_permissions() {
        // Note: This test depends on the Permission struct implementation
        // Assuming it correctly formats permissions
        let permissions = vec![
            Permission::Select("published=true OR user=$auth.id".into()),
            Permission::Update("user=$auth.id OR $auth.role=\"admin\"".into()),
        ];

        let stmt = DefineFieldStatement::new()
            .name("email")
            .table_name("user")
            .permissions(permissions);

        // This assertion should be updated based on your actual Permission formatting
        assert!(stmt.to_string().contains("DEFINE FIELD email ON TABLE user PERMISSIONS"));
        assert!(stmt.to_string().contains("FOR select"));
        assert!(stmt.to_string().contains("FOR update"));
    }

    #[test]
    fn test_complex_field() {
        let stmt = DefineFieldStatement::new()
            .name("email")
            .table_name("user")
            .column_type("string")
            .assert("string::is::email($value)")
            .value("string::lowercase($value)")
            .readonly()
            .comment("User's email address");

        assert_eq!(
            stmt.to_string(),
            "DEFINE FIELD email ON TABLE user TYPE string ASSERT string::is::email($value) VALUE string::lowercase($value) READONLY COMMENT \"User's email address\";"
        );
    }

    #[test]
    fn test_field_with_id_name() {
        let stmt = DefineFieldStatement::new()
            .name("id")
            .table_name("user")
            .column_type("string");

        // Fields named "id" should be ignored
        assert_eq!(stmt.to_string(), "");
    }

    #[test]
    fn test_missing_table() {
        let stmt = DefineFieldStatement::new()
            .name("email");

        assert!(stmt.build().is_err());
    }

    #[test]
    fn test_null_field() {
        let stmt = DefineFieldStatement::new()
            .name("description")
            .table_name("user")
            .column_type("string")
            .null();

        assert_eq!(stmt.to_string(), "DEFINE FIELD description ON TABLE user TYPE string|null ;");
    }
}
