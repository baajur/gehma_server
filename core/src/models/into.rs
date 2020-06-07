use crate::models::dao::*;
use crate::models::dto::*;

impl Into<UserDto> for UserDao {
    fn into(self) -> UserDto {
        UserDto {
            id: self.id,
            tele_num: self.tele_num.clone(),
            led: self.led,
            country_code: self.country_code.clone(),
            description: self.description.clone(),
            changed_at: self.changed_at,
            profile_picture: self.profile_picture.clone(),
            hash_tele_num: self.hash_tele_num.clone(),
            xp: self.xp,
            client_version: self.client_version,
            access_token: Some(self.access_token),
            firebase_token: self.firebase_token,
            session_token: None,
        }
    }
}

impl Into<BlacklistDto> for BlacklistDao {
    fn into(self) -> BlacklistDto {
        BlacklistDto {
            id: self.id,
            created_at: self.created_at,
            hash_blocker: self.hash_blocker.clone(),
            hash_blocked: self.hash_blocked,
        }
    }
}

impl Into<UsageStatisticEntryDto> for UsageStatisticEntryDao {
    fn into(self) -> UsageStatisticEntryDto {
        UsageStatisticEntryDto {
            id: self.id,
            tele_num: self.tele_num.clone(),
            created_at: self.created_at,
        }
    }
}

impl Into<AnalyticDto> for AnalyticDao {
    fn into(self) -> AnalyticDto {
        AnalyticDto {
            id: self.id,
            description: self.description.clone(),
            tele_num: self.tele_num.clone(),
            led: self.led,
            created_at: self.created_at,
        }
    }
}

impl Into<EventDto> for EventDao {
    fn into(self) -> EventDto {
        EventDto {
            name: self.name,
            description: self.description,
            opening: self.opening,
            href: self.href,
            id: self.id,
        }
    }
}

impl Into<VoteDto> for VoteDao {
    fn into(self) -> VoteDto {
        VoteDto {
            hash_tele_num: self.hash_tele_num,
            event_id: self.event_id,
        }
    }
}
