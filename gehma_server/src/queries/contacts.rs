use core::errors::ServiceError;
use core::models::dto::*;
use uuid::Uuid;
use mockall::*;

type IResult<K> = Result<K, ServiceError>;

#[automock]
pub trait PersistentContactsDao {
    fn create<'a>(
        &self,
        id: &Uuid,
        _user: &UserDto,
        contacts: &'a Vec<&'a mut PayloadUserDto>,
    ) -> IResult<()>;
    fn get_contacts(&self, user: &UserDto) -> IResult<Vec<ContactDto>>;
}
