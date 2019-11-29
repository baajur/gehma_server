use actix_web::web;
use uuid::Uuid;

use crate::auth::Auth;
use crate::Pool;
use core::errors::ServiceError;
use core::models::User;

use crate::routes::push_notification::Payload;

//FIXME move to user's controller
pub(crate) fn update_token_handler(
    uid: String,
    payload: Payload,
    pool: web::Data<Pool>,
    access_token: &String,
    _auth: web::Data<Auth>,
) -> Result<(), ServiceError> {
    let parsed = Uuid::parse_str(&uid)?;

    let user: Result<User, ServiceError> =
        get_user_by_id!(parsed, access_token, _auth.into_inner(), &pool);

    user?;

    crate::queries::push_notification::update_token_query(parsed, payload.token, &pool)
}
