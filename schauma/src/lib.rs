extern crate diesel;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate web_contrib;

pub(crate) mod controllers;
pub(crate) mod queries;
pub mod routes;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
