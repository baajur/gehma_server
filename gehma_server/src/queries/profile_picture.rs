use mockall::*;
use core::models::dao::*;
use core::errors::ServiceError;

type IResult<K> = Result<K, ServiceError>;

#[automock]
pub trait PersistentProfilePictureDao {
    fn get_all(&self, user: &UserDao) -> IResult<Vec<ProfilePictureDao>>;
}
