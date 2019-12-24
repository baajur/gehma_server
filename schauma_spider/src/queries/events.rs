use crate::Pool;
use actix_web::web;
use core::errors::ServiceError;
use core::models::{Event};
use diesel::{prelude::*, PgConnection};

use log::{error, info};

pub(crate) fn query_all_by_city(
    param_city: &str,
    pool: &web::Data<Pool>,
) -> Result<Vec<Event>, ::core::errors::ServiceError> {
    info!("queries/events/query_all_by_city");

    use core::schema::events::dsl::{events, city};

    let conn: &PgConnection = &pool.get().unwrap();

    events
        .filter(city.eq(param_city))
        .load::<Event>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot parse into Event".into()))
        .and_then(|result| {
            Ok(result)
        })
}
