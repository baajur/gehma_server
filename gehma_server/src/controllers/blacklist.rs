use web_contrib::auth::Auth;
use actix_web::{web};
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

use core::errors::ServiceError;
use core::models::{Blacklist, PhoneNumber, User};

use crate::Pool;

use crate::routes::blacklist::{PostData};

pub(crate) fn get_entry(
    blocker: &str,
    pool: web::Data<Pool>,
    firebase_uid: &String,
    _auth: web::Data<Auth>,
) -> Result<Vec<Blacklist>, ServiceError> {
    let blocker = Uuid::parse_str(blocker)?;

    let user : Result<User, ServiceError> = get_user_by_id!(
        blocker,
        &firebase_uid,
        _auth.into_inner(),
        &pool
    );

    user?;

    let bl = crate::queries::blacklist::get_query(blocker, pool)?;

    Ok(bl)
}

pub(crate) fn create_entry(
    blocker: &str,
    data: &PostData,
    pool: web::Data<Pool>,
    firebase_uid: &String,
    _auth: web::Data<Auth>,
) -> Result<Blacklist, ServiceError> {
    use core::schema::users::dsl::{id, users};

    let blocker2 = Uuid::parse_str(blocker)?;

    let user : Result<User, ServiceError> = get_user_by_id!(
        blocker2,
        &firebase_uid,
        _auth.into_inner(),
        &pool
    );

    user?;

    let blocked = PhoneNumber::my_from(&data.blocked, &data.country_code)?;

    //dbg!(&blocked);
    //dbg!(&blocker);

    let conn: &PgConnection = &pool.get().unwrap();

    let myusers = users
        .filter(id.eq(blocker2))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot find user".into()))?;

    let user = myusers
        .first()
        .cloned()
        .ok_or_else(|| ServiceError::BadRequest("No user found with given uid".into()))?;

    let tel = PhoneNumber::my_from(&user.tele_num, &data.country_code)?;
    let b = crate::queries::blacklist::create_query(&tel, &blocked, pool)?;

    //dbg!(&b);
    Ok(b)
}

pub(crate) fn delete_entry(
    blocker: &str,
    data: &PostData,
    pool: web::Data<Pool>,
    firebase_uid: &String,
    _auth: web::Data<Auth>,
) -> Result<(), ServiceError> {
    use core::schema::users::dsl::{id, users};

    let blocker2 = Uuid::parse_str(blocker)?;

    let user : Result<User, ServiceError>  = get_user_by_id!(
        blocker2,
        &firebase_uid,
        _auth.into_inner(),
        &pool
    );

    user?;

    let blocked = PhoneNumber::my_from(&data.blocked, &data.country_code)?;

    let conn: &PgConnection = &pool.get().unwrap();

    let myusers = users
        .filter(id.eq(blocker2))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot find user".into()))?;

    let user = myusers
        .first()
        .cloned()
        .ok_or_else(|| ServiceError::BadRequest("No user found with given uid".into()))?;

    let tel = PhoneNumber::my_from(&user.tele_num, &data.country_code)?;

    crate::queries::blacklist::delete_query(&tel, &blocked, pool)
}
