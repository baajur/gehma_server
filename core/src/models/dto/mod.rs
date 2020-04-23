use super::HashedTeleNum;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrappedUserDto {
    pub hash_tele_num: HashedTeleNum,
    pub name: String,
    pub user: Option<UserDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDto {
    pub id: Uuid,
    pub tele_num: String,
    pub led: bool,
    pub country_code: String,
    pub description: String,
    pub changed_at: chrono::NaiveDateTime,
    pub profile_picture: String,
    pub hash_tele_num: String,
    pub xp: i32,
    pub client_version: String,
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
    pub fn new(name: impl Into<String>, blocked: bool, user: UserDto) -> Self {
        Self {
            name: name.into(),
            user,
            blocked,
        }
    }
}
