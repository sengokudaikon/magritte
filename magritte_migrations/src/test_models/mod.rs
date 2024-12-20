use magritte::*;
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

#[derive(Index, Serialize, Deserialize, strum::EnumIter)]
pub enum UserV1Indexes {

}

#[derive(Event, Serialize, Deserialize, strum::EnumIter)]
pub enum UserV1Events {

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

#[derive(Index, Serialize, Deserialize, strum::EnumIter)]
pub enum UserV2Indexes {

}

#[derive(Event, Serialize, Deserialize, strum::EnumIter)]
pub enum UserV2Events {

}

#[derive(Clone, Serialize, Deserialize, Table)]
#[table(name = "products", overwrite)]
pub struct ProductV1 {
    id: SurrealId<Self>,
    #[column(type = "string")]
    name: String,
    #[column(type = "number")]
    quantity: i32,
    #[column(type = "number")]
    price: f64,
    #[column(type = "string")]
    sku: String,
    #[column(type = "any")]
    metadata: serde_json::Value,
    status: String
}

impl HasId for ProductV1 {
    fn id(&self) -> SurrealId<Self> {
        self.id.clone()
    }
}

#[derive(Event, Serialize, Deserialize, strum::EnumIter)]
pub enum ProductV1Events {
    #[event(
        name = "created",
        when = "var:before==NONE",
        then = "UPDATE products SET status = 'pending';
            CREATE log SET
            order = var:value.id,
            action     = 'product' + ' ' + var:event.lowercase(),
            old_status  = '',
            new_status  = var:after.status ?? 'pending',
            at         = time::now()
            "
    )]
    ProductCreated,
}

#[derive(Index, Serialize, Deserialize, strum::EnumIter)]
pub enum ProductV1Indexes {
}

#[derive(Table, Serialize, Deserialize, Clone)]
#[table(name = "orders", overwrite)]
pub struct OrderV1 {
    id: SurrealId<Self>,
    #[column(type = "string")]
    name: String,
    #[column(type = "number")]
    quantity: i32,
    #[column(type = "number")]
    price: f64,
    #[column(type = "string")]
    sku: String,
    #[column(type = "any")]
    metadata: serde_json::Value,
    status: String,
    product: RecordRef<ProductV1>
}

impl HasId for OrderV1 {
    fn id(&self) -> SurrealId<Self> {
        self.id.clone()
    }
}

#[derive(Index, Serialize, Deserialize, strum::EnumIter)]
pub enum OrderV1Indexes {
    #[index(
        name = "status_idx",
        fields = [status],
        comment = "Index on order status"
    )]
    StatusIdx
}

#[derive(Event, Serialize, Deserialize, strum::EnumIter)]
pub enum OrderV1Events {
}