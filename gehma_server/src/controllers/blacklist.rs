use actix_web::web;
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

use core::errors::ServiceError;
use core::models::dto::*;
use core::models::dao::*;
use core::models::PhoneNumber;

use crate::Pool;

use crate::routes::blacklist::PostData;
use crate::persistence::user::PersistentUserDao;
use crate::persistence::blacklist::PersistentBlacklistDao;

pub(crate) fn get_entry(
    blocker: &str,
    access_token: &str,
    user_dao: web::Data<&dyn PersistentUserDao>,
    blacklist_dao: web::Data<&dyn PersistentBlacklistDao>,
) -> Result<Vec<BlacklistDto>, ServiceError> {
    let blocker = Uuid::parse_str(blocker)?;

    let user: Result<UserDto, ServiceError> =
        get_user_by_id!(user_dao, &blocker, access_token.to_string());

    user?;

    let bl = blacklist_dao.get_ref().get(blocker)?;

    Ok(bl)
}

pub(crate) fn create_entry(
    blocker: &str,
    data: &PostData,
    access_token: &str,
    user_dao: web::Data<&dyn PersistentUserDao>,
    blacklist_dao: web::Data<&dyn PersistentBlacklistDao>,
    pool: web::Data<Pool>,
) -> Result<BlacklistDto, ServiceError> {
    use core::schema::users::dsl::{hash_tele_num, id, users};

    let blocker2 = Uuid::parse_str(blocker)?;

    let user: Result<UserDto, ServiceError> =
        get_user_by_id!(user_dao, &blocker2, access_token.to_string());

    user?;

    //TODO refactor to dao
    let conn: &PgConnection = &pool.get().unwrap();

    let myusers = users
        .filter(id.eq(blocker2))
        .load::<UserDao>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot find user".into()))?;

    let user = myusers
        .first()
        .cloned()
        .ok_or_else(|| ServiceError::BadRequest("No user found with given uid".into()))?;

    let contact = users
        .filter(hash_tele_num.eq(data.blocked.clone()))
        .load::<UserDao>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot find user".into()))?
        .first()
        .cloned()
        .ok_or_else(|| ServiceError::BadRequest("No user found with given hash".into()))?;

    let blocked = PhoneNumber::my_from(&contact.tele_num, &contact.country_code)?;

    let tel = PhoneNumber::my_from(&user.tele_num, &data.country_code)?;
    let b = blacklist_dao.create(&tel, &blocked)?;

    Ok(b)
}

pub(crate) fn delete_entry(
    blocker: &str,
    data: &PostData,
    access_token: &str,
    user_dao: web::Data<&dyn PersistentUserDao>,
    blacklist_dao: web::Data<&dyn PersistentBlacklistDao>,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use core::schema::users::dsl::{id, users, hash_tele_num};

    let blocker2 = Uuid::parse_str(blocker)?;

    let user: Result<UserDto, ServiceError> =
        get_user_by_id!(user_dao, &blocker2, access_token.to_string());

    user?;

    //TODO refactor to dao
    let conn: &PgConnection = &pool.get().unwrap();

    let myusers = users
        .filter(id.eq(blocker2))
        .load::<UserDao>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot find user".into()))?;

    let user = myusers
        .first()
        .cloned()
        .ok_or_else(|| ServiceError::BadRequest("No user found with given uid".into()))?;

    let tel = PhoneNumber::my_from(&user.tele_num, &data.country_code)?;

    let contact = users
        .filter(hash_tele_num.eq(data.blocked.clone()))
        .load::<UserDao>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot find user".into()))?
        .first()
        .cloned()
        .ok_or_else(|| ServiceError::BadRequest("No user found with given hash".into()))?;

    let blocked = PhoneNumber::my_from(&contact.tele_num, &contact.country_code)?;

    blacklist_dao.get_ref().delete(&tel, &blocked)
}
