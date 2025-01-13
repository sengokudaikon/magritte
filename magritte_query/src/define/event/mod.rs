//! Event definition functionality for SurrealDB.
//!
//! This module provides functionality to define events in SurrealDB that are triggered
//! after changes (create, update, delete) to records in a table. Events have access to
//! the state of the record before (`$before`) and after (`$after`) the change.
//!
//! See [SurrealDB Event Documentation](https://docs.surrealdb.com/docs/surrealql/statements/define/event)
//!
//! # Example
//!
//! ```rust
//! use magritte_query::define::*;
//!
//! // Define an event that logs email changes
//! let stmt = Define::event()
//!     .name("email_change")
//!     .table("user")
//!     .when("$before.email != $after.email")
//!     .then("CREATE log SET user = $this, action = 'email_changed', old_email = $before.email, new_email = $after.email")
//!     .comment("Log email changes")
//!     .build()
//!     .unwrap();
//! ```
//!
//! # Requirements
//!
//! - Authentication as root owner/editor, namespace owner/editor, or database owner/editor
//! - Selected namespace and database before using the statement
//! - Note: Events are not triggered during data import operations

use crate::database::{QueryType, SurrealDB};
use anyhow::{anyhow, bail};
use std::fmt::Display;
use tracing::{error, info};

/// Event types that can trigger an event
#[derive(Clone, Debug, PartialEq)]
pub enum EventType {
    /// Triggered when a new record is created
    Create,
    /// Triggered when a record is updated
    Update,
    /// Triggered when a record is deleted
    Delete,
    /// Triggered for any change (create, update, delete)
    Any,
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Create => write!(f, "$event = \"CREATE\""),
            EventType::Update => write!(f, "$event = \"UPDATE\""),
            EventType::Delete => write!(f, "$event = \"DELETE\""),
            EventType::Any => write!(
                f,
                "$event = \"CREATE\" OR $event = \"UPDATE\" OR $event = \"DELETE\""
            ),
        }
    }
}

/// Statement for defining an event in SurrealDB
#[derive(Clone, Debug, Default)]
pub struct DefineEventStatement {
    pub(crate) name: Option<String>,
    pub(crate) table: Option<String>,
    pub(crate) when: Option<String>,
    pub(crate) then: Option<String>,
    pub(crate) event_type: Option<EventType>,
    pub(crate) overwrite: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) comment: Option<String>,
}

impl DefineEventStatement {
    /// Creates a new empty event definition statement
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the event name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the table name
    pub fn table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
        self
    }

    /// Sets the event type (CREATE, UPDATE, DELETE, or ANY)
    pub fn event_type(mut self, event_type: EventType) -> Self {
        self.event_type = Some(event_type);
        self
    }

    /// Sets the WHEN condition for the event
    pub fn when(mut self, condition: impl Into<String>) -> Self {
        self.when = Some(cleanup(condition.into()));
        self
    }

    /// Sets the THEN action for the event
    pub fn then(mut self, action: impl Into<String>) -> Self {
        self.then = Some(cleanup(action.into()));
        self
    }

    /// Sets the OVERWRITE clause
    pub fn overwrite(mut self) -> Self {
        self.overwrite = true;
        self
    }

    /// Sets the IF NOT EXISTS clause
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    /// Adds a comment to the event definition
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Builds the event definition SQL statement
    pub fn build(&self) -> anyhow::Result<String> {
        // For empty/placeholder events, return empty string
        if self.name.is_none() || self.name.as_ref().map_or(true, |n| n.is_empty()) {
            return Ok(String::new());
        }

        let name = self.name.as_ref().unwrap();
        let table = self
            .table
            .as_ref()
            .ok_or_else(|| anyhow!("Table name is required"))?;
        if table.is_empty() {
            bail!("Table name is required");
        }
        if self.then.is_none() {
            bail!("Event action (THEN) is required");
        }

        let mut stmt = String::new();
        stmt.push_str("DEFINE EVENT ");

        if self.if_not_exists {
            stmt.push_str("IF NOT EXISTS ");
        } else if self.overwrite {
            stmt.push_str("OVERWRITE ");
        }

        stmt.push_str(name);
        stmt.push_str(" ON TABLE ");
        stmt.push_str(table);

        // Combine event type with custom when condition if both are present
        if let Some(event_type) = &self.event_type {
            stmt.push_str(" WHEN ");
            stmt.push_str(&event_type.to_string());
            if let Some(when) = &self.when {
                stmt.push_str(" AND (");
                stmt.push_str(when);
                stmt.push(')');
            }
        } else if let Some(when) = &self.when {
            stmt.push_str(" WHEN ");
            stmt.push_str(when);
        }

        if let Some(then) = &self.then {
            stmt.push_str(" THEN { ");
            stmt.push_str(then);
            stmt.push_str(" }");
        }

        if let Some(comment) = &self.comment {
            stmt.push_str(&format!(" COMMENT \"{}\"", comment));
        }

        stmt.push(';');
        Ok(stmt)
    }

    /// Executes the event definition statement on the database
    pub async fn execute(self, conn: &SurrealDB) -> anyhow::Result<Vec<serde_json::Value>> {
        conn.execute(self.build()?, vec![], QueryType::Schema, None).await
    }
}

/// Cleans up event expressions by handling variable references and string escaping
fn cleanup(s: String) -> String {
    s.replace("var:", "$")
        .replace("\"", "")
        .replace("\\\"", "\"")
        .replace("\\'", "'")
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\r", "\r")
}

impl Display for DefineEventStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_event() {
        let stmt = DefineEventStatement::new().build().unwrap();
        assert_eq!(stmt, "");

        let stmt = DefineEventStatement::new()
            .name("")
            .table("user")
            .then("CREATE log SET user = $this")
            .build()
            .unwrap();
        assert_eq!(stmt, "");
    }

    #[test]
    fn test_basic_event() {
        let stmt = DefineEventStatement::new()
            .name("test")
            .table("user")
            .then("CREATE log SET user = $this")
            .build()
            .unwrap();
        assert_eq!(
            stmt,
            "DEFINE EVENT test ON TABLE user THEN { CREATE log SET user = $this };"
        );
    }

    #[test]
    fn test_event_with_when() {
        let stmt = DefineEventStatement::new()
            .name("email_change")
            .table("user")
            .when("$before.email != $after.email")
            .then("CREATE log SET user = $this, action = 'email_changed'")
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE EVENT email_change ON TABLE user WHEN $before.email != $after.email THEN { CREATE log SET user = $this, action = 'email_changed' };");
    }

    #[test]
    fn test_event_with_type() {
        let stmt = DefineEventStatement::new()
            .name("user_created")
            .table("user")
            .event_type(EventType::Create)
            .then("CREATE notification SET message = 'New user created'")
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE EVENT user_created ON TABLE user WHEN $event = \"CREATE\" THEN { CREATE notification SET message = 'New user created' };");
    }

    #[test]
    fn test_event_with_type_and_when() {
        let stmt = DefineEventStatement::new()
            .name("admin_created")
            .table("user")
            .event_type(EventType::Create)
            .when("$after.role = 'admin'")
            .then("CREATE notification SET message = 'New admin created'")
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE EVENT admin_created ON TABLE user WHEN $event = \"CREATE\" AND ($after.role = 'admin') THEN { CREATE notification SET message = 'New admin created' };");
    }

    #[test]
    fn test_event_with_comment() {
        let stmt = DefineEventStatement::new()
            .name("test")
            .table("user")
            .then("CREATE log SET user = $this")
            .comment("Test event")
            .build()
            .unwrap();
        assert_eq!(stmt, "DEFINE EVENT test ON TABLE user THEN { CREATE log SET user = $this } COMMENT \"Test event\";");
    }

    #[test]
    fn test_event_with_overwrite() {
        let stmt = DefineEventStatement::new()
            .name("test")
            .table("user")
            .then("CREATE log SET user = $this")
            .overwrite()
            .build()
            .unwrap();
        assert_eq!(
            stmt,
            "DEFINE EVENT OVERWRITE test ON TABLE user THEN { CREATE log SET user = $this };"
        );
    }

    #[test]
    fn test_event_if_not_exists() {
        let stmt = DefineEventStatement::new()
            .name("test")
            .table("user")
            .then("CREATE log SET user = $this")
            .if_not_exists()
            .build()
            .unwrap();
        assert_eq!(
            stmt,
            "DEFINE EVENT IF NOT EXISTS test ON TABLE user THEN { CREATE log SET user = $this };"
        );
    }

    #[test]
    fn test_cleanup_macro_vars() {
        let stmt = DefineEventStatement::new()
            .name("created")
            .table("order")
            .when("var:before==NONE")
            .then(
                r#"UPDATE orders SET status = 'pending';
                CREATE log SET
                order = var:value.id,
                action = 'order' + ' ' + var:event.lowercase(),
                old_status = '',
                new_status = var:after.status ?? 'pending',
                at = time::now()"#,
            )
            .build()
            .unwrap();
        assert!(stmt.contains("$before==NONE"));
        assert!(stmt.contains("$value.id"));
        assert!(stmt.contains("$event.lowercase()"));
        assert!(stmt.contains("$after.status"));
        assert!(stmt.contains("THEN {"));
        assert!(stmt.contains("};"));
    }

    #[test]
    fn test_multi_statement_then() {
        let stmt = DefineEventStatement::new()
            .name("complex_event")
            .table("user")
            .when("$before.status != $after.status")
            .then(
                r#"
                LET $old_status = $before.status;
                LET $new_status = $after.status;
                CREATE notification SET 
                    user = $this,
                    message = 'Status changed from ' + $old_status + ' to ' + $new_status;
                UPDATE stats SET 
                    status_changes += 1 
                WHERE user = $this
            "#,
            )
            .build()
            .unwrap();
        assert!(stmt.contains("THEN {"));
        assert!(stmt.contains("LET $old_status"));
        assert!(stmt.contains("LET $new_status"));
        assert!(stmt.contains("CREATE notification"));
        assert!(stmt.contains("UPDATE stats"));
        assert!(stmt.contains("};"));
    }

    #[test]
    fn test_missing_table() {
        let stmt = DefineEventStatement::new()
            .name("test")
            .then("CREATE log SET user = $this")
            .build();
        assert!(stmt.is_err());
    }

    #[test]
    fn test_empty_table() {
        let stmt = DefineEventStatement::new()
            .name("test")
            .table("")
            .then("CREATE log SET user = $this")
            .build();
        assert!(stmt.is_err());
    }

    #[test]
    fn test_missing_then() {
        let stmt = DefineEventStatement::new()
            .name("test")
            .table("user")
            .build();
        assert!(stmt.is_err());
    }
}
