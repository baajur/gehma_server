use crate::queries::*;
use actix_web::{web};
use core::errors::ServiceError;
use core::models::dto::*;
use uuid::Uuid;

use crate::get_user_by_id;

pub(crate) fn get_entries(
    uid: &str,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    mark_seen: bool,
) -> Result<Vec<BroadcastElementDto>, ServiceError> {
    let parsed = Uuid::parse_str(uid)?;

    let user = get_user_by_id!(user_dao, &parsed);
    let user = user?;

    let elements = user_dao
        .into_inner()
        .get_latest_broadcast(&user, mark_seen)?
        .into_iter()
        .map(|w| w.into())
        .collect();

    Ok(elements)
}
