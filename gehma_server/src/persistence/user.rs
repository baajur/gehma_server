use chrono::{DateTime, Local};
use core::errors::ServiceError;
use core::models::dto::*;
use core::models::PhoneNumber;
use uuid::Uuid;
use mockall::*;

type IResult<K> = Result<K, ServiceError>;

#[automock]
pub trait PersistentUserDao {
    fn get_by_tele_num(
        &self,
        tele: &PhoneNumber,
        my_access_token: String,
    ) -> IResult<UserDto>;
    fn get_by_id(&self, id: &Uuid, my_access_token: String) -> IResult<UserDto>;
    fn get_contacts(&self, user: &UserDto) -> IResult<Vec<ContactDto>>;
    fn create_analytics_for_user(&self, user: &UserDto) -> IResult<AnalyticDto>;
    fn create(
        &self,
        tele_num: &PhoneNumber,
        country_code: &str,
        client_version: &str,
        access_token: &str,
    ) -> IResult<UserDto>;

    fn create_usage_statistics_for_user(&self, user: &UserDto) -> IResult<UsageStatisticEntryDto>;

    fn update_user(
        &self,
        id: &Uuid,
        user: &UpdateUserDto,
        current_time: DateTime<Local>,
    ) -> IResult<UserDto>;
    fn update_profile_picture(&self, user: &UserDto) -> IResult<()>;
    fn update_token(&self, id: &Uuid, token: String) -> IResult<()>;
}
