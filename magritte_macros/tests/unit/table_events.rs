use crate::{Order, OrderColumns, User};
use magritte::prelude::*;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};

// Test table events
#[derive(Event, Serialize, Deserialize, strum::EnumIter)]
pub enum OrderEvents {
    #[event(
        name = "created",
        when = "var:before==NONE",
        then = "UPDATE orders SET status = 'pending';
            CREATE log SET
            order = var:value.id,
            action     = 'order' + ' ' + var:event.lowercase(),
            old_status  = '',
            new_status  = var:after.status ?? 'pending',
            at         = time::now()
            "
    )]
    Created,
}


#[test]
fn test_complex_column_derives() {
    // Test relationship column definitions
    let user_def = OrderColumns::User.def();
    assert_eq!(user_def.column_type(), "record<users>");
    assert!(user_def.assert().is_some());
    assert!(!user_def.is_nullable());

    // Test array relationship column
    let items_def = OrderColumns::Items.def();
    assert_eq!(items_def.column_type(), "array<record<products>>");

    // Test enum-like value constraints
    let status_def = OrderColumns::Status.def();
    assert_eq!(status_def.column_type(), "string");
    let status_stmt = status_def.to_statement();
    assert!(status_stmt.contains("VALUE"));
    assert!(status_stmt.contains("pending"));
    assert!(status_stmt.contains("delivered"));

    // Test nested object column
    let shipping_def = OrderColumns::ShippingInfo.def();
    assert_eq!(shipping_def.column_type(), "object");
    assert!(shipping_def.is_flexible());

    // Test decimal type column
    let total_def = OrderColumns::Total.def();
    assert_eq!(total_def.column_type(), "decimal");
    assert_eq!(total_def.assert(), Some("value >= 0"));

    // Test datetime column
    let created_def = OrderColumns::CreatedAt.def();
    assert_eq!(created_def.column_type(), "datetime");
}

#[test]
fn test_column_statement_variations() {
    // Test relationship field statement
    let user_stmt = OrderColumns::User.def().to_statement();
    assert!(user_stmt.contains("DEFINE FIELD user ON TABLE orders"));
    assert!(user_stmt.contains("TYPE record<users>"));
    assert!(user_stmt.contains("ASSERT value != NONE"));

    // Test array relationship statement
    let items_stmt = OrderColumns::Items.def().to_statement();
    assert!(items_stmt.contains("DEFINE FIELD items ON TABLE orders"));
    assert!(items_stmt.contains("TYPE array<record<products>>"));

    // Test enum-like value constraint statement
    let status_stmt = OrderColumns::Status.def().to_statement();
    assert!(status_stmt.contains("DEFINE FIELD status ON TABLE orders"));
    let expected_value = "pending|processing|shipped|delivered";
    assert!(status_stmt.contains(format!("VALUE {}", expected_value).as_str()));
}

#[test]
fn test_table_events() {
    // Test event definitions
    let created = OrderEvents::Created;
    assert_eq!(OrderEvents::table_name(), "orders");
    assert_eq!(created.event_name(), "created");
    // Test event statements
    let created_stmt = created.to_statement();
    assert!(created_stmt.contains("DEFINE EVENT created ON TABLE orders"));
    assert!(created_stmt.contains("WHEN $before==NONE"));
    assert!(created_stmt.contains("THEN UPDATE orders SET status = 'pending';"));
}

#[test]
fn test_table_with_events() {
    // Test table with events
    let _order = Order::new(
        "1",
        "2023-01-01T00:00:00Z".to_string(),
        User::new("2", "Jane", "jdoe@me.com").as_record(),
        vec![],
        100.0,
        serde_json::json!({}),
        "pending".to_string(),
    );

    // Test event triggers
    let events = Order::events().collect::<Vec<_>>();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_name(), "created");
}
