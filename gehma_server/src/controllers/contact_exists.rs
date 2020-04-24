use actix_web::web;
use uuid::Uuid;
use core::errors::ServiceError;
use core::models::dto::*;

//use crate::routes::contact_exists::{PayloadUser, ResponseUser};

use crate::persistence::contact_exists::PersistentContactExistsDao;
use crate::persistence::user::PersistentUserDao;

pub(crate) fn get_entry(
    uid: &str,
    country_code: &str,
    phone_numbers: &mut Vec<PayloadUserDto>,
    access_token: &str,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    contact_exists_dao: web::Data<Box<dyn PersistentContactExistsDao>>,
) -> Result<Vec<WrappedUserDto>, ServiceError> {
    let parsed = Uuid::parse_str(uid)?;

    let user: Result<UserDto, ServiceError> =
        get_user_by_id!(user_dao, &parsed, access_token.to_string());

    let users = contact_exists_dao
        .get_ref()
        .get(&parsed, &user?, phone_numbers, country_code)?;

    Ok(users)
}
