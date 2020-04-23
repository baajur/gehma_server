use actix_web::web;
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

use core::errors::ServiceError;
use core::models::PhoneNumber;
use core::models::dao::*;
use core::models::dto::*;

use crate::Pool;

use log::{error, info};

pub(crate) fn get_query(
    sblocker: Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<BlacklistDto>, ServiceError> {
    info!("queries/blacklist/get_query");
    use core::schema::blacklist::dsl::{blacklist, hash_blocker};
    use core::schema::users::dsl::{id, users};

    let conn: &PgConnection = &pool.get().unwrap();

    let user = users
        .filter(id.eq(sblocker))
        .load::<UserDao>(conn)
        .map_err(|_db_err| ServiceError::BadRequest("Invalid User".into()))?
        .first()
        .cloned()
        .ok_or_else(|| ServiceError::BadRequest("No user found".into()))?;

    blacklist
        .filter(hash_blocker.eq(user.hash_tele_num))
        .load::<BlacklistDao>(conn)
        .map(|w| w.into_iter().map(|k| k.into()).collect())
        .map_err(|_db_err| ServiceError::BadRequest("Invalid User".into()))
}

pub(crate) fn create_query(
    blocker: &PhoneNumber,
    blocked: &PhoneNumber,
    pool: web::Data<Pool>,
) -> Result<BlacklistDto, ServiceError> {
    info!("queries/blacklist/create_query");
    use core::schema::blacklist::dsl::blacklist;

    let conn: &PgConnection = &pool.get().unwrap();
    let new_inv: BlacklistDao = BlacklistDao::my_from(blocker, blocked);

    //println!("{:?}", new_inv);

    diesel::insert_into(blacklist)
        .values(&new_inv)
        .get_result::<BlacklistDao>(conn)
        .map(|w| w.into())
        .map_err(|_db_error| {
            error!("{:?}", _db_error);
            ServiceError::BadRequest("Cannot insert into blacklist".into())
        })

}

pub(crate) fn delete_query(
    sblocker: &PhoneNumber,
    sblocked: &PhoneNumber,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    info!("queries/blacklist/delete_query");
    use core::schema::blacklist::dsl::{blacklist, hash_blocked, hash_blocker};
    use data_encoding::HEXUPPER;
    use ring::digest;

    let conn: &PgConnection = &pool.get().unwrap();

    let b1 =
        HEXUPPER.encode(digest::digest(&digest::SHA256, sblocker.to_string().as_bytes()).as_ref());

    let b2 =
        HEXUPPER.encode(digest::digest(&digest::SHA256, sblocked.to_string().as_bytes()).as_ref());

    //FIXME #34
    let target = blacklist
        .filter(hash_blocker.eq(b1))
        .filter(hash_blocked.eq(b2));

    diesel::delete(target)
        .execute(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot delete blacklist".into()))?;

    Ok(())
}
