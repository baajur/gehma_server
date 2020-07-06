use crate::models::dao::*;
use crate::models::dto::*;

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

impl Into<ProfilePictureDto> for ProfilePictureDao {
    fn into(self) -> ProfilePictureDto {
        ProfilePictureDto {
            id: self.id,
            path: self.path,
        }
    }
}

impl Into<BroadcastElementDto> for BroadcastElementDao {
    fn into(self) -> BroadcastElementDto {
        BroadcastElementDto {
            id: self.id,
            display_user: self.display_user,
            originator_user_id: self.originator_user_id,
            text: self.text,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
