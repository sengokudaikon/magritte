use magritte::{HasId, SurrealId, Table};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Table)]
#[table(name = "users", overwrite)]
pub struct UserV1 {
    id: SurrealId<Self>,
    #[column(type = "string")]
    name: String,
    #[column(type = "number")]
    age: i32,
}
impl HasId for UserV1 {
    fn id(&self) -> SurrealId<Self> {
        self.id.clone()
    }
}

// Assume UserV2 adds a new column "email"
#[derive(Clone, Serialize, Deserialize, Table)]
#[table(name = "users", overwrite)]
pub struct UserV2 {
    id: SurrealId<Self>,
    #[column(type = "string")]
    name: String,
    #[column(type = "number")]
    age: i32,
    #[column(type = "string")]
    email: String,
}
impl HasId for UserV2 {
    fn id(&self) -> SurrealId<Self> {
        self.id.clone()
    }
}
