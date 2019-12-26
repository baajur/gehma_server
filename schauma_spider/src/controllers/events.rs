use crate::Pool;
use actix_web::{error::BlockingError, error::PayloadError, web};
use core::errors::ServiceError;
use core::models::{Event};
use diesel::{prelude::*, PgConnection};
use futures::future::{err, Either};
use futures::stream::Stream;
use futures::Future;
use uuid::Uuid;

use log::{error, info};
use std::io::Write;

use crate::queries::events::query_all_by_city;

/// Get all events by city
pub(crate) fn get_all_by_city(
    city: &str,
    pool: web::Data<Pool>,
) -> Result<Vec<Event>, ServiceError> {
    query_all_by_city(city, &pool)
}