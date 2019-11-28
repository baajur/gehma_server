use actix_web::{web};
use uuid::Uuid;

use crate::Pool;
use core::models::User;
use core::errors::ServiceError;
use crate::auth::Auth;

use crate::routes::push_notification::Payload;

//FIXME move to user's controller
pub(crate) fn update_token_handler(
    uid: String,
    payload: Payload,
    pool: web::Data<Pool>,
    firebase_uid: &String,
    auth: web::Data<Auth>,
) -> Result<(), ServiceError> {
    let parsed = Uuid::parse_str(&uid)?;

    let user : Result<User, ServiceError> = get_user_by_id!(parsed, firebase_uid, auth.into_inner(), &pool);

    user?;

    crate::queries::push_notification::update_token_query(parsed, payload.token, &pool)
}
