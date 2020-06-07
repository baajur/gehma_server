use crate::errors::ServiceError;
use crate::models::dto::*;
use crate::schema::*;
use crate::utils::phonenumber_to_international;

use diesel::sql_types::{Nullable, Text, Uuid};
use diesel::{Queryable, QueryableByName};

use data_encoding::HEXUPPER;
use ring::digest;

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Queryable,
    Insertable,
    Clone,
    Identifiable,
    AsChangeset,
    Eq,
    PartialEq,
    QueryableByName,
)]
#[table_name = "users"]
pub struct UserDao {
    pub id: uuid::Uuid,
    pub tele_num: String,
    pub led: bool,
    pub created_at: chrono::NaiveDateTime,
    pub country_code: String,
    pub description: String,
    pub changed_at: chrono::NaiveDateTime,
    pub client_version: String,
    pub firebase_token: Option<String>,
    pub profile_picture: String,
    pub access_token: String,
    pub hash_tele_num: HashedTeleNum,
    pub xp: i32,
}

macro_rules! hash {
    ($col:expr) => {
        HashedTeleNum(HEXUPPER.encode(digest::digest(&digest::SHA256, $col.as_bytes()).as_ref()))
    };
}

impl UserDao {
    pub fn my_from(tele_num: &str, country_code: &str, version: &str, access_token: &str) -> Self {
        UserDao {
            id: uuid::Uuid::new_v4(),
            tele_num: tele_num.to_string(),
            led: false,
            created_at: chrono::Local::now().naive_local(),
            changed_at: chrono::Local::now().naive_local(),
            country_code: country_code.to_string(),
            description: "".to_string(),
            client_version: version.to_string(),
            firebase_token: None,
            profile_picture: "".to_string(),
            access_token: access_token.to_string(),
            xp: 0,
            hash_tele_num: hash!(tele_num),
        }
    }

    pub fn apply_update(mut self, user: &UpdateUserDto, time: chrono::NaiveDateTime) -> Self {
        self.led = user.led;
        self.description = user.description.clone();
        self.client_version = user.client_version.clone();
        self.changed_at = time;

        self
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone, Identifiable)]
#[table_name = "blacklist"]
//#[belongs_to(User, foreign_key = "hash_blocker")]
//#[belongs_to(User, foreign_key = "hash_blocked")]
pub struct BlacklistDao {
    pub id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub hash_blocker: HashedTeleNum,
    pub hash_blocked: HashedTeleNum,
}

impl BlacklistDao {
    pub fn my_from(blocker: &PhoneNumber, blocked: &PhoneNumber) -> Self {
        BlacklistDao {
            id: uuid::Uuid::new_v4(),
            created_at: chrono::Local::now().naive_local(),
            hash_blocker: hash!(blocker.to_string()),
            hash_blocked: hash!(blocked.to_string()),
        }
    }
}

//TODO extract
#[derive(Debug)]
pub struct PhoneNumber(phonenumber::PhoneNumber);

impl PhoneNumber {
    /*
    pub fn to_string(&self) -> String {
        use phonenumber::Mode;

        format!("{}", self.0.format().mode(Mode::International)).replace(" ", "")
    }
    */

    pub fn my_from(raw: &str, cc: &str) -> Result<Self, ServiceError> {
        Ok(PhoneNumber(phonenumber_to_international(raw, cc)?))
    }
}

use std::fmt;
impl fmt::Display for PhoneNumber {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use phonenumber::Mode;
        fmt.write_str(&format!("{}", self.0.format().mode(Mode::International)).replace(" ", ""))
            .expect("fmt failed");
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Identifiable)]
#[table_name = "analytics"]
pub struct AnalyticDao {
    pub id: i32,
    pub tele_num: String,
    pub led: bool,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "analytics"]
pub struct InsertAnalyticDao {
    pub tele_num: String,
    pub led: bool,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
}

impl AnalyticDao {
    pub fn my_from(user: &UserDao) -> InsertAnalyticDao {
        InsertAnalyticDao {
            tele_num: user.tele_num.clone(),
            led: user.led,
            description: user.description.clone(),
            created_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Identifiable)]
#[table_name = "usage_statistics"]
pub struct UsageStatisticEntryDao {
    pub id: i32,
    pub tele_num: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "usage_statistics"]
pub struct InsertUsageStatisticEntryDao {
    pub tele_num: String,
    pub created_at: chrono::NaiveDateTime,
}

impl UsageStatisticEntryDao {
    pub fn my_from(user: &UserDao) -> InsertUsageStatisticEntryDao {
        InsertUsageStatisticEntryDao {
            tele_num: user.tele_num.clone(),
            created_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(
    Debug, Serialize, Deserialize, Queryable, Clone, Associations, QueryableByName, Insertable,
)]
#[table_name = "contacts"]
#[belongs_to(UserDao, foreign_key = "from_id")]
#[belongs_to(ContactDao, foreign_key = "target_hash_tele_num")]
pub struct ContactDao {
    pub from_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub name: String,
    pub target_hash_tele_num: HashedTeleNum,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Insertable, Associations)]
#[table_name = "contacts"]
#[belongs_to(UserDao, foreign_key = "from_id")]
#[belongs_to(ContactDao, foreign_key = "target_hash_tele_num")]
pub struct ContactInsertDao {
    pub from_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub name: String,
    pub target_hash_tele_num: HashedTeleNum,
}

/// Database response for users which
/// should be send a push notification
#[derive(Debug, Deserialize, Clone, Queryable, QueryableByName)]
pub struct ContactPushNotificationDao {
    #[sql_type = "Uuid"]
    pub from_id: uuid::Uuid,
    #[sql_type = "Text"]
    pub name: String,
    #[sql_type = "Nullable<Text>"]
    pub firebase_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Identifiable, AsChangeset, Eq, PartialEq, QueryableByName, Queryable)]
#[table_name = "events"]
pub struct EventDao {
    pub name: String,
    pub description: String,
    pub opening: chrono::NaiveDateTime,
    pub country: String,
    pub city: String,
    pub addr: String,
    pub href: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub changed_at: chrono::NaiveDateTime,
    pub id: i32,
}

/*
impl ContactDao {
    pub fn my_from(name: String, user: &UserDao, target_tele_num: String) -> ContactInsertDao {
        ContactInsertDao {
            from_id: user.id,
            target_tele_num: target_tele_num.clone(),
            created_at: chrono::Local::now().naive_local(),
            name,
            target_hash_tele_num: hash!(target_tele_num),
        }
    }
}
*/
