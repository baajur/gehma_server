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
    ) -> IResult<UserDao>;

    /// Get user by hash_tele_num without access_token
    fn get_by_hash_tele_num_unsafe(
        &self,
        hash_tele_num: &HashedTeleNum,
    ) -> IResult<UserDao>;

    fn get_by_id(&self, id: &Uuid) -> IResult<UserDao>;

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
    fn update_profile_picture(&self, id: Uuid, user: &UpdateProfilePictureDto) -> IResult<()>;
    fn update_token(&self, id: &Uuid, token: String) -> IResult<()>;

    /// Get the profile picture's path
    fn get_profile_picture(&self, user: &UserDao) -> IResult<String>;

    /// User `originator_user` creates a broadcast entry for the `disaplay_user`
    fn create_broadcast_entry(&self, originator_user: &UserDao, display_user: &HashedTeleNum, text: &String) -> IResult<()>;

    /// Get's the latest broadcast elements, which wasn't seen yet, for the user `user_id`.
    /// `mark_seen` determines if it is only peeked
    fn get_latest_broadcast(&self, user: &UserDao, mark_seen: bool) -> IResult<Vec<BroadcastElementDao>>;

    /// Updates all unseen elements to seen.
    fn update_latest_broadcast(&self, user: &UserDao) -> IResult<()>;
}
