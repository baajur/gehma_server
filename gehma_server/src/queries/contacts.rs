use core::errors::ServiceError;
use core::models::dto::*;
use core::models::dao::*;
use uuid::Uuid;
use mockall::*;

type IResult<K> = Result<K, ServiceError>;

#[automock]
pub trait PersistentContactsDao {
    fn create<'a>(
        &self,
        id: &Uuid,
        _user: &UserDao,
        contacts: &'a Vec<&'a mut PayloadUserDto>,
    ) -> IResult<()>;
    fn get_contacts(&self, user: &UserDao) -> IResult<Vec<ContactDto>>;
}
