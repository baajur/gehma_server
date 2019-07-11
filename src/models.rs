use super::schema::*;
use diesel::{r2d2::ConnectionManager, PgConnection};

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone)]
#[table_name = "users"]
pub struct User {
    pub id: uuid::Uuid,
    pub tele_num: String,
    pub led: bool,
    pub created_at: chrono::NaiveDateTime,
}

impl<T> From<T> for User where T: Into<String> {
    fn from(e: T) -> Self {
        User {
            id: uuid::Uuid::new_v4(),
            tele_num: e.into().replace("+", "").replace(" ", "").trim().to_string(),
            led: false,
            created_at: chrono::Local::now().naive_local()
        }
    }
}
