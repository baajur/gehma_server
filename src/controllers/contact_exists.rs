use actix_web::{web};
use uuid::Uuid;

use crate::Pool;
use core::errors::ServiceError;

use core::models::User;
use crate::auth::FirebaseDatabaseConfiguration;

use crate::routes::contact_exists::{ResponseUser, PayloadUser};

pub(crate) fn get_entry(
    uid: &str,
    country_code: &str,
    phone_numbers: &mut Vec<PayloadUser>,
    pool: web::Data<Pool>,
    firebase_uid: &String,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> Result<Vec<ResponseUser>, ServiceError> {
    let parsed = Uuid::parse_str(uid)?;

    let user : Result<User, ServiceError> = authenticate_user_by_uid!(parsed, firebase_uid, firebase_config.into_inner(), &pool);

    user?;

    let users =
        crate::queries::contact_exists::get_query(parsed, phone_numbers, country_code, pool)?;

    Ok(users)
}
