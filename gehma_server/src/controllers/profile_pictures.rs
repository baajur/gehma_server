use crate::get_user_by_id;
use crate::queries::*;
use actix_web::web;
use core::errors::ServiceError;
use core::models::dto::*;
use log::trace;
use uuid::Uuid;

pub(crate) fn get_all_profile_pictures(
    id: &str,
    user_dao: web::Data<Box<dyn PersistentUserDao>>,
    p_dao: web::Data<Box<dyn PersistentProfilePictureDao>>,
) -> Result<Vec<ProfilePictureDto>, ServiceError> {
    trace!("controllers/profile_picture/get_all_profile_pictures");
    let parsed = Uuid::parse_str(id)?;

    let user = get_user_by_id!(user_dao, &parsed);

    Ok(p_dao
        .get_ref()
        .get_all(&user?)?
        .into_iter()
        .map(|w| w.into())
        .collect())
}
