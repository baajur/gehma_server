use crate::queries::*;
use core::errors::ServiceError;
use core::models::dao::*;
use core::models::dto::*;
use mockall::*;
use std::sync::Arc;
use uuid::Uuid;

type IResult<K> = Result<K, ServiceError>;

#[automock]
pub trait PersistentContactsDao {
    fn create<'a>(
        &self,
        id: &Uuid,
        _user: &UserDao,
        contacts: &'a Vec<&'a mut PayloadUserDto>,
    ) -> IResult<()>;

    fn get_contacts(
        &self,
        user: &UserDao,
        user_dao: Arc<Box<dyn PersistentUserDao>>,
    ) -> IResult<Vec<ContactDto>>;
}
