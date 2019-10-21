use super::schema::*;
use crate::errors::InternalError;
use crate::utils::phonenumber_to_international;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone, Identifiable, AsChangeset, Eq, PartialEq)]
#[table_name = "users"]
pub struct User {
    pub id: uuid::Uuid,
    pub tele_num: String,
    pub led: bool,
    pub created_at: chrono::NaiveDateTime,
    pub country_code: String,
    pub description: String,
    pub is_autofahrer: bool,
    pub changed_at: chrono::NaiveDateTime,
    pub client_version: String,
    pub firebase_token: Option<String>,
    pub profile_picture: String,
}

/* We don't want to expose all user's data to everyone. That's why this struct
 * is a minimal version of `User` with only essential fields.
 * */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DowngradedUser {
    pub tele_num: String,
    pub led: bool,
    pub country_code: String,
    pub description: String,
    pub is_autofahrer: bool,
    pub changed_at: chrono::NaiveDateTime,
    pub profile_picture: String,
}

impl User {
    pub fn my_from(e: &str, country_code: &str, version: &str) -> Self {
        User {
            id: uuid::Uuid::new_v4(),
            tele_num: e.to_string(),
            led: false,
            created_at: chrono::Local::now().naive_local(),
            changed_at: chrono::Local::now().naive_local(),
            country_code: country_code.to_string(),
            description: "".to_string(),
            is_autofahrer: false,
            client_version: version.to_string(),
            firebase_token: None,
            profile_picture: "".to_string(),
        }
    }

    pub fn downgrade(&self) -> DowngradedUser {
        DowngradedUser {
            tele_num: self.tele_num.clone(),
            led: self.led,
            country_code: self.country_code.clone(),
            description: self.description.clone(),
            is_autofahrer: self.is_autofahrer,
            changed_at: self.changed_at,
            profile_picture: self.profile_picture.clone()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone, Identifiable)]
#[table_name = "blacklist"]
//#[belongs_to(User, foreign_key="blocker")]
pub struct Blacklist {
    pub id: uuid::Uuid,
    pub blocker: String,
    pub blocked: String,
    pub created_at: chrono::NaiveDateTime,
}

impl Blacklist {
    pub fn my_from(blocker: &PhoneNumber, blocked: &PhoneNumber) -> Self {
        Blacklist {
            id: uuid::Uuid::new_v4(),
            blocker: blocker.to_string(),
            blocked: blocked.to_string(),
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

#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Identifiable)]
#[table_name = "analytics"]
pub struct Analytic {
    pub id: i32,
    pub tele_num: String,
    pub led: bool,
    pub is_autofahrer: bool,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "analytics"]
pub struct InsertAnalytic {
    pub tele_num: String,
    pub led: bool,
    pub is_autofahrer: bool,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
}

impl Analytic {
    pub fn my_from(user: &User) -> InsertAnalytic {
        InsertAnalytic {
            tele_num: user.tele_num.clone(),
            led: user.led,
            is_autofahrer: user.is_autofahrer,
            description: user.description.clone(),
            created_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Identifiable)]
#[table_name = "usage_statistics"]
pub struct UsageStatisticEntry {
    pub id: i32,
    pub tele_num: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "usage_statistics"]
pub struct InsertUsageStatisticEntry {
    pub tele_num: String,
    pub created_at: chrono::NaiveDateTime,
}

impl UsageStatisticEntry {
    pub fn my_from(user: &User) -> InsertUsageStatisticEntry {
        InsertUsageStatisticEntry {
            tele_num: user.tele_num.clone(),
            created_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Identifiable, Associations)]
#[belongs_to(User, foreign_key="from_tele_num")]
#[table_name = "contacts"]
pub struct Contact {
    pub id: i32,
    pub from_id: uuid::Uuid,
    pub target_tele_num: String,
    pub created_at: chrono::NaiveDateTime,
    pub name: String,
    pub from_tele_num: String,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Clone, Insertable, Associations)]
#[belongs_to(User, foreign_key="from_tele_num")]
#[table_name = "contacts"]
pub struct ContactInsert {
    pub from_id: uuid::Uuid,
    pub target_tele_num: String,
    pub created_at: chrono::NaiveDateTime,
    pub name: String,
    pub from_tele_num: String,
}

impl Contact {
    pub fn my_from(name: String, user: &User, target_tele_num: String) -> ContactInsert {
        ContactInsert {
            from_id: user.id,
            target_tele_num,
            created_at: chrono::Local::now().naive_local(),
            name,
            from_tele_num: user.tele_num.clone(),
        }
    }
}
