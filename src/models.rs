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

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone)]
#[table_name = "blacklist"]
pub struct Blacklist {
    pub blocker: String,
    pub blocked: String,
    pub created_at: chrono::NaiveDateTime,
}

impl Blacklist {
    pub fn my_from(blocker: &String, blocked: &String) -> Self {
        Blacklist {
            blocker: blocker.to_string(),
            blocked: blocked.to_string(),
            created_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(Debug)]
pub struct PhoneNumber(String);

impl FromStr for PhoneNumber {
    type Err = crate::errors::InternalError;

    fn from(e: String) -> Result<Self, Self::Err> {
        unimplemented!("")
    }
}

impl PhoneNumber {
    pub fn to_string() -> String {
        unimplemented!()
    }
}
