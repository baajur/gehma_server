pub mod user;

use crate::Pool;
use actix_web::web;

type DbPool = web::Data<Pool>;
