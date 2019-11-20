use actix_web::{web};
use uuid::Uuid;

use crate::Pool;
use core::models::User;
use core::errors::ServiceError;
use crate::auth::FirebaseDatabaseConfiguration;

use crate::routes::push_notification::Payload;

pub(crate) fn update_token_handler(
    uid: String,
    payload: Payload,
    pool: web::Data<Pool>,
    firebase_uid: &String,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> Result<(), ServiceError> {
    let parsed = Uuid::parse_str(&uid)?;

    let user : Result<User, ServiceError> = authenticate_user_by_uid!(parsed, firebase_uid, firebase_config.into_inner(), &pool);

    user?;

    crate::queries::push_notification::update_token_query(parsed, payload.token, &pool)
}
