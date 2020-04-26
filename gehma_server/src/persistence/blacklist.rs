use core::errors::ServiceError;
use core::models::dto::*;
use core::models::PhoneNumber;
use uuid::Uuid;
use mockall::*;

type IResult<K> = Result<K, ServiceError>;

#[automock]
pub trait PersistentBlacklistDao {
    fn get(&self, sblocker: Uuid) -> IResult<Vec<BlacklistDto>>;
    fn create(&self, blocker: &PhoneNumber, blocked: &PhoneNumber) -> IResult<BlacklistDto>;

    fn delete(&self, blocker: &HashedTeleNum, blocked: &HashedTeleNum) -> IResult<()>;
}
