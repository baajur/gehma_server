use chrono::{DateTime, Local};
use core::errors::ServiceError;
use core::models::dto::*;
use core::models::dao::*;
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
    ) -> IResult<UserDao>;

    /// Get user by hash_tele_num without access_token
    fn get_by_hash_tele_num_unsafe(
        &self,
        hash_tele_num: &HashedTeleNum,
    ) -> IResult<UserDao>;

    fn get_by_id(&self, id: &Uuid, my_access_token: String) -> IResult<UserDao>;
    fn get_by_id_unsafe(&self, id: &Uuid) -> IResult<UserDao>;

    fn create_analytics_for_user(&self, user: &UserDao) -> IResult<AnalyticDao>;
    fn create(
        &self,
        tele_num: &PhoneNumber,
        country_code: &str,
        client_version: &str,
        access_token: &str,
    ) -> IResult<UserDao>;

    fn create_usage_statistics_for_user(&self, user: &UserDao) -> IResult<UsageStatisticEntryDao>;

    fn update_user(
        &self,
        id: &Uuid,
        user: &UpdateUserDto,
        current_time: DateTime<Local>,
    ) -> IResult<(UserDao, Vec<ContactPushNotificationDao>)>;
    fn update_profile_picture(&self, user: &UserDao) -> IResult<()>;
    fn update_token(&self, id: &Uuid, token: String) -> IResult<()>;
}
