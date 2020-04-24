use core::errors::ServiceError;
use core::models::dto::*;
use core::models::PhoneNumber;
use uuid::Uuid;

type IResult<K> = Result<K, ServiceError>;

pub trait PersistentBlacklistDao {
    fn get(&self, sblocker: Uuid) -> IResult<Vec<BlacklistDto>>;
    fn create(&self, blocker: &PhoneNumber, blocked: &PhoneNumber) -> IResult<BlacklistDto>;

    fn delete(&self, blocker: &PhoneNumber, blocked: &PhoneNumber) -> IResult<()>;
}
