use crate::errors::ServiceError;
use crate::models::dto::*;
use crate::schema::*;
use crate::utils::phonenumber_to_international;

use diesel::sql_types::{Nullable, Text, Uuid};
use diesel::{Associations, Queryable, QueryableByName};

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
    pub access_token: String,
    pub hash_tele_num: HashedTeleNum,
    pub xp: i32,
    pub profile_picture: i32,
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
            profile_picture: 1, //DEFAULT
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

    pub fn into(self, profile_picture: String) -> crate::models::dto::UserDto {
        UserDto {
            id: self.id,
            tele_num: self.tele_num.clone(),
            led: self.led,
            country_code: self.country_code.clone(),
            description: self.description.clone(),
            changed_at: self.changed_at,
            profile_picture: profile_picture.to_string(),
            hash_tele_num: self.hash_tele_num.clone(),
            xp: self.xp,
            client_version: self.client_version,
            access_token: Some(self.access_token),
            firebase_token: self.firebase_token,
            session_token: None,
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
    #[sql_type = "Text"]
    pub target_hash_tele_num: HashedTeleNum,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Identifiable,
    AsChangeset,
    Eq,
    PartialEq,
    QueryableByName,
    Queryable,
)]
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

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    AsChangeset,
    Eq,
    PartialEq,
    Queryable,
    QueryableByName,
    Insertable,
)]
#[table_name = "votes"]
pub struct VoteDao {
    pub hash_tele_num: HashedTeleNum,
    pub event_id: i32,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    AsChangeset,
    Eq,
    PartialEq,
    Queryable,
    QueryableByName,
    Identifiable,
    Insertable,
)]
#[table_name = "profile_pictures"]
pub struct ProfilePictureDao {
    pub id: i32,
    pub path: String,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    AsChangeset,
    Eq,
    PartialEq,
    Queryable,
    QueryableByName,
    Identifiable,
)]
#[table_name = "broadcast"]
pub struct BroadcastElementDao {
    pub id: i32,
    /// User who created it
    pub originator_user_id: uuid::Uuid,
    pub text: String,
    pub is_seen: bool,
    pub updated_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    /// User for whom, it will be display
    pub display_user: HashedTeleNum,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "broadcast"]
pub struct InsertBroadcastElementDao {
    /// User who created it
    pub originator_user_id: uuid::Uuid,
    pub text: String,
    pub is_seen: bool,
    pub updated_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    /// User for whom, it will be display
    pub display_user: HashedTeleNum,
}


impl BroadcastElementDao {
    pub fn my_from(self, contact: &ContactDto) -> BroadcastElementDto {
        BroadcastElementDto {
            id: self.id,
            display_user: self.display_user,
            originator_user: contact.clone(),
            text: self.text,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Associations, QueryableByName, Queryable, Clone)]
#[table_name = "invitation"]
#[belongs_to(UserDao, foreign_key = "originator_user_id")]
#[belongs_to(InvitationDao, foreign_key = "id")]
pub struct InvitationDao {
    pub id: i32,
    pub originator_user_id: uuid::Uuid,
    pub edit_text: String,
    pub edit_time: chrono::NaiveDateTime,
    pub original_text: String,
    pub original_time: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "invitation"]
pub struct InsertInvitationDao {
    pub originator_user_id: uuid::Uuid,
    pub edit_text: String,
    pub edit_time: chrono::NaiveDateTime,
    pub original_text: String,
    pub original_time: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Associations, Insertable, Queryable, QueryableByName, Clone)]
#[table_name = "invitation_members"]
#[belongs_to(UserDao, foreign_key = "user_id")]
#[belongs_to(InvitationDao, foreign_key = "inv_id")]
pub struct InvitationMemberDao {
    pub inv_id: i32,
    pub user_id: uuid::Uuid,
    pub is_seen: bool,
    pub state: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}


