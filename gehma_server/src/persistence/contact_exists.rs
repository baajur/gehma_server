use core::errors::ServiceError;
use core::models::dto::*;
use uuid::Uuid;

type IResult<K> = Result<K, ServiceError>;

//TODO rename
pub trait PersistentContactExistsDao {
    fn get(
        &self,
        id: &Uuid,
        _user: &UserDto,
        phone_numbers: &mut Vec<PayloadUserDto>,
        _country_code: &str,
    ) -> IResult<Vec<WrappedUserDto>>;
}
