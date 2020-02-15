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

impl Event {
    pub fn generate(name: impl Into<String>, description: impl Into<String>, country: impl Into<String>, city: impl Into<String>, addr: impl Into<String>) -> Self {
        Event {
            id: uuid::Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            opening: chrono::Local::now().naive_local(),
            closing: chrono::Local::now().naive_local() + (chrono::Duration::days(1)),
            country: country.into(),
            city: city.into(),
            addr: addr.into(),
            href: None,
            created_at: chrono::Local::now().naive_local(),
            changed_at: chrono::Local::now().naive_local(),
        }
    }
}

pub struct DatasourceGenericEvent {
    pub name: String,
    pub description: String,
    pub opening: chrono::NaiveDateTime,
    pub closing: chrono::NaiveDateTime,
    pub country: String,
    pub city: String,
    pub addr: String,
    pub href: Option<String>,
}

impl From<Event> for DatasourceGenericEvent {
    fn from(event: Event) -> Self {
        DatasourceGenericEvent {
            name: event.name,
            description: event.description,
            opening: event.opening,
            closing: event.closing,
            country: event.country,
            city: event.city,
            addr: event.addr,
            href: event.href,
        }
    }
}

/*
impl Into<DatasourceGenericEvent> for Event {
    fn into(self) -> DatasourceGenericEvent {
        DatasourceGenericEvent {
            name: self.name,
            description: self.description,
            opening: self.opening,
            closing: self.closing,
            country: self.country,
            city: self.city,
            addr: self.addr,
            href: self.href,
        }
    }
}
*/
