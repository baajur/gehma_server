use actix_web::web;
use uuid::Uuid;

use crate::Pool;
use core::errors::ServiceError;

use crate::auth::Auth;
use core::models::User;

use crate::routes::contact_exists::{PayloadUser, ResponseUser};

pub(crate) fn get_entry(
    uid: &str,
    country_code: &str,
    phone_numbers: &mut Vec<PayloadUser>,
    pool: web::Data<Pool>,
    access_token: &String,
    _auth: web::Data<Auth>,
) -> Result<Vec<ResponseUser>, ServiceError> {
    let parsed = Uuid::parse_str(uid)?;

    let user: Result<User, ServiceError> =
        get_user_by_id!(parsed, access_token, _auth.into_inner(), &pool);

    user?;

    let users =
        crate::queries::contact_exists::get_query(parsed, phone_numbers, country_code, pool)?;

    Ok(users)
}
