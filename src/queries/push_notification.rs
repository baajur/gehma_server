use actix_web::web;
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

use crate::Pool;
use core::errors::ServiceError;

use log::{error, info};

pub(crate) fn update_token_query(
    uid: Uuid,
    token: String,
    pool: &web::Data<Pool>,
) -> Result<(), ServiceError> {
    info!("queries/push_notification/update_token_query");
    use core::schema::users::dsl::*;
    let conn: &PgConnection = &pool.get().unwrap();

    let target = users.filter(id.eq(uid));

    diesel::update(target)
        .set(
            firebase_token.eq(Some(token))
            //disabled 29.11.2019
            //changed_at.eq(chrono::Local::now().naive_local()),
        )
        .execute(conn)
        .map_err(|_db_error| {
            error!("db_error {}", _db_error);
            ServiceError::BadRequest("Updating state failed".into())
        })?;

    Ok(())
}
