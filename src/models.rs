use super::schema::*;
use crate::errors::InternalError;
use crate::utils::phonenumber_to_international;
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
            tele_num: e.to_string(),
            led: false,
            created_at: chrono::Local::now().naive_local(),
            country_code: country_code.to_string(),
            description: "".to_string(),
            is_autofahrer: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone)]
#[table_name = "blacklist"]
pub struct Blacklist {
    pub blocker: uuid::Uuid,
    pub blocked: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
}

impl Blacklist {
    pub fn my_from(blocker: uuid::Uuid, blocked: uuid::Uuid) -> Self {
        Blacklist {
            blocker: blocker,
            blocked: blocked,
            created_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(Debug)]
pub struct PhoneNumber(phonenumber::PhoneNumber);

impl PhoneNumber {
    pub fn to_string(&self) -> String {
        use phonenumber::Mode;

        format!("{}", self.0.format().mode(Mode::International)).replace(" ", "")
    }

    pub fn my_from(raw: &str, cc: &str) -> Result<Self, InternalError> {
        Ok(PhoneNumber(phonenumber_to_international(
            raw,
            cc,
        )?))
    }
}
