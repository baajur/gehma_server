use diesel::backend::Backend;
use diesel::deserialize;
use diesel::serialize::{self, Output, ToSql};
use diesel::types::FromSql;
use std::io::Write;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, AsExpression, FromSqlRow, PartialEq, Eq, Hash)]
#[sql_type = "diesel::sql_types::Text"]
pub struct HashedTeleNum(pub String);

impl<DB> ToSql<diesel::sql_types::Text, DB> for HashedTeleNum
where
    DB: Backend,
    String: ToSql<diesel::sql_types::Text, DB>,
{
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        <String as ToSql<diesel::sql_types::Text, DB>>::to_sql(&self.0, out)
    }
}

impl<DB> FromSql<diesel::sql_types::Text, DB> for HashedTeleNum
where
    DB: Backend,
    String: FromSql<diesel::sql_types::Text, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        <String as FromSql<diesel::sql_types::Text, DB>>::from_sql(bytes).map(HashedTeleNum)
    }
}

use std::fmt;
impl fmt::Display for HashedTeleNum {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.0).expect("fmt failed");
        Ok(())
    }
}

//FIXME merge WrappedUserDto with UserDto

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrappedUserDto {
    pub hash_tele_num: HashedTeleNum,
    pub name: String,
    pub user: Option<UserDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
//FIXME `created_at` missing
pub struct UserDto {
    pub id: Uuid,
    pub tele_num: String,
    pub led: bool,
    pub country_code: String,
    pub description: String,
    pub changed_at: chrono::NaiveDateTime,
    pub profile_picture: String,
    pub hash_tele_num: HashedTeleNum,
    pub xp: i32,
    pub client_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    pub firebase_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PayloadNumbersDto {
    pub numbers: Vec<PayloadUserDto>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PayloadUserDto {
    pub name: String,
    pub hash_tele_num: HashedTeleNum,
}

//TODO merge!

#[derive(Debug, Serialize, Deserialize)]
pub struct PostUserDto {
    pub tele_num: String,
    pub country_code: String,
    pub client_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserDto {
    pub description: String,
    pub led: bool,
    pub client_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlacklistDto {
    pub id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub hash_blocker: HashedTeleNum,
    pub hash_blocked: HashedTeleNum,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalyticDto {
    pub id: i32,
    pub tele_num: String,
    pub led: bool,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsageStatisticEntryDto {
    pub id: i32,
    pub tele_num: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactDto {
    pub user: UserDto,
    pub name: String,
    pub blocked: bool,
}

impl ContactDto {
    pub fn new(name: impl Into<String>, blocked: bool, mut user: UserDto) -> Self {
        user.access_token = None;
        user.firebase_token = None;
        Self {
            name: name.into(),
            user,
            blocked,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RequestCodeDto {
    pub tele_num: String,
    pub country_code: String,
}

#[derive(Debug, Deserialize)]
pub struct RequestCheckCodeDto {
    pub tele_num: String,
    pub code: String,
    pub country_code: String,
    pub client_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct EventDto {
    pub name: String,
    pub description: String,
    pub opening: chrono::NaiveDateTime,
    pub href: Option<String>,
    pub id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct VoteDto {
    pub hash_tele_num: HashedTeleNum,
    pub event_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct ProfilePictureDto {
    pub id: i32,
    pub path: String
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct UpdateProfilePictureDto {
    pub profile_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BroadcastElementDto {
    pub id: i32,
    /// User for whom, it will be display
    pub display_user: HashedTeleNum, 
    /// User who created it
    pub originator_user: ContactDto,
    pub text: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
