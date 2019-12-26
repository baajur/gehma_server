use crate::Pool;
use actix_web::web;
use core::errors::ServiceError;
use core::models::{Event, DatasourceGenericEvent};
use diesel::{prelude::*, PgConnection};

use log::{info};

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

/// Checks if events in `my_events` are already in the database and if not then insert them
pub(crate) fn populate_events(my_events: &Vec<DatasourceGenericEvent>,
    pool: &web::Data<Pool>,
    ) -> Result<Vec<Event>, ::core::errors::SchaumaError> {
    info!("queries/events/query_all_by_city");

    use core::schema::events::dsl::{events, city};

    let conn: &PgConnection = &pool.get().unwrap();

    todo!("Check events if already in the database");

    /*
    let t = events
        .filter(city.eq(param_city))
        .load::<Event>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot parse into Event".into()))
        .and_then(|result| {
            Ok(result)
        });
    */

    // Insert all `my_events` into `events` when they don't already exist
    todo!("Insert");

    //TODO
    Ok(vec![])
}
