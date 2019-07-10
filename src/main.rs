#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{middleware, web, App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

mod models;
mod schema;
mod errors;
//mod invitation_handler;
//mod register_handler;
mod user_handler;
mod exists_handler;
//mod contacts_handler;
mod utils;

fn main() {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info,actix_server=info");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL expected");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool : models::Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create a pool");
    let domain : String = std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".into());

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(utils::SECRET_KEY.as_bytes())
                        .name("auth")
                        .path("/")
                        .domain(domain.as_str())
                        .max_age_time(chrono::Duration::days(1))
                        .secure(false) //FIXME
            )) 
            //.data(web::JsonConfig::default().limit(4096))
            .data(web::JsonConfig::default())
            .service(
                web::scope("/api")
                /*
                 * .service(
                    web::resource("/")
                        .route(web::get().to_async(|| actix_web::HttpResponse::Ok().finish()))
                )
                */
                .service(
                    web::resource("/user/{base_tel}")
                        .route(web::get().to_async(user_handler::get))
                        .route(web::put().to_async(user_handler::update))
                        .route(web::post().to_async(user_handler::add))
                )
                .service(
                    web::resource("/exists/{base_tel}")
                        .route(web::post().to_async(exists_handler::get))
                )
            )
        })
        .bind("10.0.0.50:3000").unwrap()
        .run().unwrap()

}
