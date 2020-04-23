use crate::schema::*;
use crate::errors::InternalError;
use crate::utils::phonenumber_to_international;
use super::HashedTeleNum;

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
    pub hash_tele_num: String,
    pub xp: i32,
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
            hash_tele_num: 
                HEXUPPER.encode(digest::digest(&digest::SHA256, tele_num.as_bytes()).as_ref()),
        }
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
            hash_blocker: HEXUPPER
                .encode(digest::digest(&digest::SHA256, blocker.to_string().as_bytes()).as_ref()),
            hash_blocked: HEXUPPER
                .encode(digest::digest(&digest::SHA256, blocked.to_string().as_bytes()).as_ref()),
        }
    }
}

//TODO extract
#[derive(Debug)]
pub struct PhoneNumber(phonenumber::PhoneNumber);

impl PhoneNumber {
    pub fn to_string(&self) -> String {
        use phonenumber::Mode;

        format!("{}", self.0.format().mode(Mode::International)).replace(" ", "")
    }

    pub fn my_from(raw: &str, cc: &str) -> Result<Self, InternalError> {
        Ok(PhoneNumber(phonenumber_to_international(raw, cc)?))
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

#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Associations, QueryableByName)]
#[table_name = "contacts"]
#[belongs_to(UserDao, foreign_key = "from_id")]
#[belongs_to(ContactDao, foreign_key = "target_hash_tele_num")]
pub struct ContactDao {
    pub from_id: uuid::Uuid,
    pub target_tele_num: String,
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
    pub target_tele_num: String,
    pub created_at: chrono::NaiveDateTime,
    pub name: String,
    pub target_hash_tele_num: HashedTeleNum,
}

impl ContactDao {
    pub fn my_from(name: String, user: &UserDao, target_tele_num: String) -> ContactInsertDao {
        ContactInsertDao {
            from_id: user.id,
            target_tele_num: target_tele_num.clone(),
            created_at: chrono::Local::now().naive_local(),
            name,
            target_hash_tele_num: HEXUPPER
                .encode(digest::digest(&digest::SHA256, target_tele_num.as_bytes()).as_ref()),
        }
    }
}
