use super::HashedTeleNum;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDto {
    pub tele_num: String,
    pub led: bool,
    pub country_code: String,
    pub description: String,
    pub changed_at: chrono::NaiveDateTime,
    pub profile_picture: String,
    pub hash_tele_num: String,
    pub xp: i32,
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
    pub from_id: uuid::Uuid,
    pub target_tele_num: String,
    pub created_at: chrono::NaiveDateTime,
    pub name: String,
    pub target_hash_tele_num: HashedTeleNum,
}
