use crate::schema::*;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone, Identifiable, AsChangeset, Eq, PartialEq)]
#[table_name = "events"]
pub struct Event {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub opening: chrono::NaiveDateTime,
    pub closing: chrono::NaiveDateTime,
    pub country: String,
    pub city: String,
    pub addr: String,
    pub href: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub changed_at: chrono::NaiveDateTime,
}
