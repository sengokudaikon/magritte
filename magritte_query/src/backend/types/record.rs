use std::marker::PhantomData;
use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use surrealdb::{sql, RecordId, RecordIdKey};
use crate::types::RecordType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordRef<T>(String, PhantomData<T>)
where
    T: RecordType;
impl<T> RecordRef<T>
where
    T: RecordType,
{
    pub fn new() -> Self {
        Self(format!("record<{}>", T::table_name()), PhantomData)
    }
}

impl<T> From<RecordRef<T>> for sql::Value
where
    T: RecordType,
{
    fn from(value: RecordRef<T>) -> Self {
        sql::Value::from(value.0)
    }
}
#[derive(Debug, Clone)]
pub struct SurrealId<T>(RecordId, PhantomData<T>)
where
    T: RecordType;
impl<T> SurrealId<T>
where
    T: RecordType,
{
    pub fn new<I: Into<RecordIdKey>>(id: I) -> Self {
        let record_id = RecordId::from((T::table_name().to_string(), id.into()));
        Self(record_id, PhantomData)
    }

    pub fn to_record_id(&self) -> &RecordId {
        &self.0
    }
    pub fn table(&self) -> &str {
        self.0.table()
    }
    pub fn id(&self) -> &RecordIdKey {
        self.0.key()
    }
}
impl<T> From<String> for SurrealId<T>
where
    T: RecordType,
{
    fn from(value: String) -> Self {
        let record_id = RecordId::from((String::from(T::table_name()), RecordIdKey::from(value)));
        Self(record_id, PhantomData)
    }
}

impl<T> std::fmt::Display for SurrealId<T>
where
    T: RecordType,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let table = self.0.table();
        let id = self.0.key().to_string();
        write!(f, "{}:{}", table, id)
    }
}
impl<T> std::ops::Deref for SurrealId<T>
where
    T: RecordType,
{
    type Target = RecordId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> PartialEq for SurrealId<T>
where
    T: RecordType,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for SurrealId<T>
where
    T: RecordType
{}

impl<T> std::hash::Hash for SurrealId<T>
where
    T: RecordType,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Serialize for SurrealId<T>
where
    T: RecordType,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for SurrealId<T>
where
    T: RecordType,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let record_id = RecordId::deserialize(deserializer)?;
        Ok(Self(record_id, PhantomData))
    }
}

impl<T> From<serde_json::Value> for SurrealId<T> where T: RecordType {
    fn from(value: serde_json::Value) -> Self {
        SurrealId::new(value.to_string())
    }
}
impl <T> From<&str> for SurrealId<T> where T: RecordType {
    fn from(value: &str) -> Self {
        SurrealId::new(value)
    }
}
impl<T> FromStr for SurrealId<T> where T: RecordType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SurrealId::new(s.to_string()))
    }
}
