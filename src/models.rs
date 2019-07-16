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
    pub country_code: String,
    pub description: String,
    pub is_autofahrer: bool,
}

impl User {
    pub fn my_from(e: &String, country_code: &String) -> Self {
        User {
            id: uuid::Uuid::new_v4(),
            tele_num: e.replace("+", "").replace(" ", "").trim().to_string(),
            led: false,
            created_at: chrono::Local::now().naive_local(),
            country_code: country_code.to_string(),
            description: "".to_string(),
            is_autofahrer: false
        }
    }
}
