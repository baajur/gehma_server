use actix_web::web;
use core::errors::ServiceError;
use core::models::dto::*;
use uuid::Uuid;

use crate::queries::*;

use crate::get_user_by_id;

pub const MAX_ALLOWED_CONTACTS: usize = 10000;

pub(crate) fn create(
    uid: &str,
    _country_code: &str,
    phone_numbers: &mut Vec<PayloadUserDto>,
    access_token: &str,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    blacklist_dao: web::Data<Box<dyn PersistentBlacklistDao>>,
    contact_dao: web::Data<Box<dyn PersistentContactsDao>>,
) -> Result<(), ServiceError> {
    let parsed = Uuid::parse_str(uid)?;

    let user = get_user_by_id!(user_dao, &parsed, access_token.to_string());

    if phone_numbers.len() >= MAX_ALLOWED_CONTACTS {
        return Err(ServiceError::BadRequest("Too many contacts.".to_string()));
    }

    let blacklists: Vec<_> = blacklist_dao
        .get_ref()
        .get(parsed)?
        .into_iter()
        .map(|w| w.hash_blocked)
        .collect();

    let filtered_phone_numbers: Vec<_> = phone_numbers
        .into_iter()
        .filter(|w| !blacklists.contains(&w.hash_tele_num))
        .collect();

    let _ = contact_dao
        .get_ref()
        .create(&parsed, &user?, &filtered_phone_numbers)?;

    Ok(())
}

pub(crate) fn get_contacts(
    uid: &str,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    blacklist_dao: web::Data<Box<dyn PersistentBlacklistDao>>,
    contact_dao: web::Data<Box<dyn PersistentContactsDao>>,
    access_token: &str,
) -> Result<Vec<ContactDto>, ServiceError> {
    let parsed = Uuid::parse_str(uid)?;

    let user = get_user_by_id!(user_dao, &parsed, access_token.to_owned());

    let user = user?;

    //TODO change this to a HashSet for performance
    let blacklists: Vec<_> = blacklist_dao
        .get_ref()
        .get(parsed)?
        .into_iter()
        .map(|w| w.hash_blocked)
        .collect();

    let mut contacts = contact_dao.get_ref().get_contacts(&user)?;

    contacts
        .iter_mut()
        .filter(|w| blacklists.contains(&w.user.hash_tele_num))
        .for_each(|w| {
            w.blocked = true;
            w.user.led = false;
            w.user.description = "".to_string();
        });

    //TODO make parallel?
    for mut contact in contacts.iter_mut().filter(|w| !w.blocked) {
        let other_blacklists: Vec<_> = blacklist_dao
            .get_ref()
            .get(contact.user.id)?
            .into_iter()
            .map(|w| w.hash_blocked)
            .filter(|w| w == &user.hash_tele_num) //someone blocked me
            .collect();

        if other_blacklists.len() > 0 {
            contact.blocked = false; // I cannot if someone blocked me
            contact.user.led = false;
            contact.user.description = "".to_string();
        }
    }

    Ok(contacts)
}
