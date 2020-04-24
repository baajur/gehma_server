pub mod user;
pub mod blacklist;
pub mod contact_exists;

use crate::Pool;
use actix_web::web;

type DbPool = web::Data<Pool>;
