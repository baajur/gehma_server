use actix_web::web;
use uuid::Uuid;

use core::errors::ServiceError;
use core::models::dto::*;
use core::models::PhoneNumber;

use crate::queries::*;
use crate::routes::blacklist::PostData;
use log::debug;

use crate::get_user_by_id;

pub(crate) fn get_entry(
    blocker: &str,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    blacklist_dao: web::Data<Box<dyn PersistentBlacklistDao>>,
) -> Result<Vec<BlacklistDto>, ServiceError> {
    let blocker = Uuid::parse_str(blocker)?;

    let user =
        get_user_by_id!(user_dao, &blocker);

    user?;

    let bl = blacklist_dao
        .get_ref()
        .get(blocker)?
        .into_iter()
        .map(|w| w.into())
        .collect();

    Ok(bl)
}

pub(crate) fn create_entry(
    blocker: &str,
    data: &PostData,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    blacklist_dao: web::Data<Box<dyn PersistentBlacklistDao>>,
) -> Result<BlacklistDto, ServiceError> {
    debug!("controllers/blacklist/create_entry");
    let blocker2 = Uuid::parse_str(blocker)?;

    let user =
        get_user_by_id!(user_dao, &blocker2);

    let contact = user_dao
        .get_ref()
        .get_by_hash_tele_num_unsafe(&HashedTeleNum(data.hash_blocked.clone()))?;

    let blocked = PhoneNumber::my_from(&contact.tele_num, &contact.country_code)?;

    let tel = PhoneNumber::my_from(&user?.tele_num, &data.country_code)?;

    let b = blacklist_dao.create(&tel, &blocked)?.into();

    Ok(b)
}

pub(crate) fn delete_entry(
    blocker: &str,
    data: &PostData,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    blacklist_dao: web::Data<Box<dyn PersistentBlacklistDao>>,
) -> Result<(), ServiceError> {
    let blocker2 = Uuid::parse_str(blocker)?;

    let user =
        get_user_by_id!(user_dao, &blocker2);

    blacklist_dao.get_ref().delete(
        &user?.hash_tele_num,
        &HashedTeleNum(data.hash_blocked.clone()),
    )
}
