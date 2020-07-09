use crate::queries::*;
use actix_web::web;
use core::errors::ServiceError;
use core::models::dto::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::get_user_by_id;

pub(crate) fn get_entries(
    uid: &str,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    contact_dao: web::Data<Box<dyn PersistentContactsDao>>,
    mark_seen: bool,
) -> Result<Vec<BroadcastElementDto>, ServiceError> {
    let parsed = Uuid::parse_str(uid)?;

    let user = get_user_by_id!(user_dao, &parsed);
    let user = user?;

    let user_dao = user_dao.into_inner();

    let contacts = contact_dao.get_contacts(&user, &user_dao)?;
    let mut lookup = HashMap::new();

    for c in contacts {
        lookup.insert(c.user.id.clone(), c);
    }

    eprintln!("lookup {:?}", lookup);

    let elements = user_dao
        .get_latest_broadcast(&user, mark_seen)?
        .into_iter()
        .filter_map(|w| {
            eprintln!("{:#?}", w.originator_user_id);
            let contact = lookup.get(&w.originator_user_id);

            if let Some(c) = contact {
                Some(w.my_from(c))
            }
            else {
                log::warn!("Filtering contact {:?}", w);
                None
            }
        })
        .collect();

    Ok(elements)
}
