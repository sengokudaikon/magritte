use magritte_query::NamedType;
use crate::entity::{HasColumns, HasEvents, HasIndexes, HasRelations};
use crate::prelude::{ColumnTrait, EdgeTrait, EventTrait, IndexTrait, RelationTrait, TableTrait};

pub trait MigrationTable {
    fn up() -> &'static str;
    fn down() -> &'static str;
}

pub trait MigrationEdge {
    fn up() -> &'static str;
    fn down() -> &'static str;
}
pub trait MigrationEdgeColumns {
    fn up() -> &'static str;
    fn down() -> &'static str;
}

pub trait MigrationTableColumns {
    fn up() -> &'static str;
    fn down() -> &'static str;
}

pub trait MigrationTableColumnsIndex {
    fn up() -> &'static str;
    fn down() -> &'static str;
}

pub trait MigrationTableColumnsIndexEvent {
    fn up() -> &'static str;
    fn down() -> &'static str;
}

pub trait MigrationTableColumnsIndexEventRelation {
    fn up() -> &'static str;
    fn down() -> &'static str;
}

// Define the Migration struct
pub struct Migration {
    script_up: String,
    script_down: String,
}

impl Migration where Self: 'static {
    // Initialize a new Migration
    pub fn new() -> Self where Self: 'static  {
        Self {
            script_up: String::new(),
            script_down: String::new(),
        }
    }

    // Add a table to the migration
    pub fn with_table<T: TableTrait>(mut self) -> Self {
        let table_def = T::to_statement();
        self.script_up.push_str(&table_def);
        self.script_up.push_str("\n");
        self
    }

    // Add an edge to the migration
    pub fn with_edge<E: EdgeTrait>(mut self) -> Self {
        let edge_def = E::to_statement();
        self.script_up.push_str(&edge_def);
        self.script_up.push_str("\n");
        self
    }

    // Add columns to the migration
    pub fn with_columns<T: HasColumns>(mut self) -> Self {
        for column in T::columns() {
            let column_def = ColumnTrait::to_statement(&column);
            self.script_up.push_str(&column_def);
            self.script_up.push_str("\n");
        }
        self
    }

    // Add indexes to the migration
    pub fn with_indexes<T: HasIndexes>(mut self) -> Self {
        for index in T::indexes() {
            let index_def =
                IndexTrait::to_statement(&index).expect("Failed to get index statement");
            self.script_up.push_str(&index_def);
            self.script_up.push_str("\n");
        }
        self
    }

    // Add events to the migration
    pub fn with_events<T: HasEvents>(mut self) -> Self {
        for event in T::events() {
            let event_def = EventTrait::to_statement(&event);
            self.script_up.push_str(&event_def);
            self.script_up.push_str("\n");
        }
        self
    }

    // Add relations to the migration
    pub fn with_relations<T: HasRelations>(mut self) -> Self {
        for relation in T::relations() {
            let relation_def = RelationTrait::to_statement(&relation);
            self.script_up.push_str(&relation_def);
            self.script_up.push_str("\n");
        }
        self
    }

    // Get the migration script for `up` migrations
    pub fn up(&'static self) -> &'static str {
        &self.script_up
    }

    // Get the migration script for `down` migrations
    pub fn down(&'static self) -> &'static str {
        &self.script_down
    }
}