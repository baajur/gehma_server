use crate::models::dao::*;
use crate::models::dto::*;

impl Into<UserDto> for UserDao {
    fn into(self) -> UserDto {
        UserDto {
            id: self.id.clone(),
            tele_num: self.tele_num.clone(),
            led: self.led,
            country_code: self.country_code.clone(),
            description: self.description.clone(),
            changed_at: self.changed_at,
            profile_picture: self.profile_picture.clone(),
            hash_tele_num: self.hash_tele_num.clone(),
            xp: self.xp,
            client_version: self.client_version.clone(),
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

impl Into<UsageStatisticEntryDto> for UsageStatisticEntryDao {
    fn into(self) -> UsageStatisticEntryDto {
        UsageStatisticEntryDto  {
            id: self.id.clone(),
            tele_num: self.tele_num.clone(),
            created_at: self.created_at.clone(),
        }
    }
}

impl Into<AnalyticDto> for AnalyticDao {
    fn into(self) -> AnalyticDto {
        AnalyticDto  {
            id: self.id.clone(),
            description: self.description.clone(),
            tele_num: self.tele_num.clone(),
            led: self.led,
            created_at: self.created_at.clone(),

        }
    }
}
