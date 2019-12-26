use crate::datasources::EventDatasourceWrapper;
use crate::Pool;
use actix_web::web;
use core::errors::{SchaumaError, ServiceError};
use core::models::Event;
use diesel::{prelude::*, PgConnection};
use chrono::prelude::*;

use log::info;

use crate::queries::events::query_all_by_city;

/// Get all events by city
pub(crate) fn get_all_by_city(
    city: &str,
    pool: web::Data<Pool>,
) -> Result<Vec<Event>, ServiceError> {
    query_all_by_city(city, &pool)
}

/// Create new events
/// This method is used in `schauma_spider`
#[allow(dead_code)]
pub(crate) fn populate_events(
    city: String,
    date: &str,
    pool: web::Data<Pool>,
    event_datasource: web::Data<EventDatasourceWrapper>,
) -> Result<Vec<Event>, ServiceError> {
    info!("controllers/events/populate_events");

    let opening = parse_date(date)?;
    let opening = NaiveDateTime::new(opening, NaiveTime::from_hms(0, 0, 0));

    // Fetch datasources
    // Get events from opening date to END (because None)
    let events = event_datasource
        .into_inner()
        .service
        .get_events(&pool.clone().into_inner(), &city, opening, None)
        .map_err(|err| ServiceError::SchaumaError(err))?;

    crate::queries::events::populate_events(&events, &pool)
        .map_err(|err| ServiceError::SchaumaError(err))
}

#[allow(dead_code)]
fn parse_date(date: &str) -> Result<chrono::NaiveDate, ServiceError> {

    NaiveDate::parse_from_str(date, "%d-%m-%Y").map_err(|_| {
        ServiceError::SchaumaError(SchaumaError::ParseError(format!(
            "{} is not a valid format",
            date
        )))
    })
}
