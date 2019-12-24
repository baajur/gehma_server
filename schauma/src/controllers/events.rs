use crate::Pool;
use actix_web::web;
use core::errors::ServiceError;
use core::models::{Event};
use diesel::{prelude::*, PgConnection};

use log::info;

use crate::queries::events::query_all_by_city;

/// Get all events by city
pub(crate) fn get_all_by_city(
    city: &str,
    pool: web::Data<Pool>,
) -> Result<Vec<Event>, ServiceError> {
    query_all_by_city(city, &pool)
}

pub(crate) fn populate_events(date: &str, _pool: web::Data<Pool>) -> Result<(), ServiceError> { 
    info!("controllers/events/populate_events");
    //Fetch datasources
    //Insert each

    let _date = parse_date(date)?;
    
    todo!();

    Ok(())
}

#[allow(dead_code)]
fn parse_date(date: &str) -> Result<chrono::NaiveDate, ServiceError> {
    use chrono::prelude::*;

    NaiveDate::parse_from_str(date, "%d-%m-%Y")
        .map_err(|_| ServiceError::ParseError(format!("{} is not a valid format", date)))
}
