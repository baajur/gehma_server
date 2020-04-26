use chrono::{DateTime, Local};
use core::errors::ServiceError;
use core::models::dto::*;
use core::models::PhoneNumber;
use uuid::Uuid;
use mockall::*;
use actix_web::web;
use crate::services::push_notifications::NotificationService;

type IResult<K> = Result<K, ServiceError>;

#[automock]
pub trait PersistentUserDao {
    fn get_by_tele_num(
        &self,
        tele: &PhoneNumber,
        my_access_token: String,
    ) -> IResult<UserDto>;

    /// Get user by hash_tele_num without access_token
    fn get_by_hash_tele_num_unsafe(
        &self,
        hash_tele_num: &HashedTeleNum,
    ) -> IResult<UserDto>;

    fn get_by_id(&self, id: &Uuid, my_access_token: String) -> IResult<UserDto>;
    fn get_by_id_unsafe(&self, id: &Uuid) -> IResult<UserDto>;

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
        notification_service: web::Data<NotificationService>,
    ) -> IResult<UserDto>;
    fn update_profile_picture(&self, user: &UserDto) -> IResult<()>;
    fn update_token(&self, id: &Uuid, token: String) -> IResult<()>;
}
