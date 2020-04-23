use crate::models::dao::*;
use crate::models::dto::*;

impl Into<UserDto> for UserDao {
    fn into(self) -> UserDto {
        UserDto {
            tele_num: self.tele_num.clone(),
            led: self.led,
            country_code: self.country_code.clone(),
            description: self.description.clone(),
            changed_at: self.changed_at,
            profile_picture: self.profile_picture.clone(),
            hash_tele_num: self.hash_tele_num.clone(),
            xp: self.xp,
        }
    }
}

impl Into<BlacklistDto> for BlacklistDao {
    fn into(self) -> BlacklistDto {
        BlacklistDto {
            id: self.id.clone(),
            created_at: self.created_at.clone(),
            hash_blocker: self.hash_blocker.clone(),
            hash_blocked: self.hash_blocked.clone(),
        }
    }
}
