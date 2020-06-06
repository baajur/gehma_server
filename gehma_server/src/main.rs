#[macro_use]
extern crate serde_derive;
extern crate web_contrib;

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::http::header;
use actix_web::{middleware as actix_middleware, web, App, HttpResponse, HttpServer};
use core::errors::{InternalServerError, ServiceError};
use log::error;
use std::path::PathBuf;

use crate::utils::*;

pub(crate) mod controllers;
pub(crate) mod queries;
pub(crate) mod ratelimits; //move to services
pub(crate) mod routes;
pub(crate) mod services;

pub(crate) mod dao_factory;

mod middleware;

mod database;
mod redis;
mod utils;

use dao_factory::*;

#[cfg(test)]
mod tests;

pub const ALLOWED_CLIENT_VERSIONS: &[&str] = &["0.5.4"];
pub const LIMIT_PUSH_NOTIFICATION_CONTACTS: usize = 128;
pub const ALLOWED_PROFILE_PICTURE_SIZE: usize = 10_000; //in Kilobytes

use crate::database::*;
use crate::redis::*;

#[actix_rt::main]
pub(crate) async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info,actix_web=info,actix_server=info");
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL expected");
    //let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL expected");

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = std::env::var("BINDING_ADDR").unwrap_or_else(|_| "localhost".to_string());

    let pool_pg = connect_pg(database_url);
    //let pool_redis = connect_redis(redis_url);

    let server = HttpServer::new(move || {
        let dao_factory = DaoFactory::new(pool_pg.clone());

        App::new()
            .data(pool_pg.clone())
            //.data(pool_redis.clone())
            //.data(get_auth())
            .data(set_testing_auth())
            .data(get_onesignal_notification_service())
            .data(get_ratelimits())
            .data(get_session_service())
            .data(dao_factory.get_user_dao())
            .data(dao_factory.get_contacts_dao())
            .data(dao_factory.get_blacklist_dao())
            .wrap(
                Cors::new()
                    .allowed_origin("http://localhost:3000")
                    .allowed_origin("https://gehma.xyz")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600)
                    .finish(),
            )
            .wrap(actix_middleware::Logger::default())
            .data(web::JsonConfig::default().limit(4048 * 1024))
            .wrap(actix_middleware::Compress::default())
            .wrap(middleware::auth::Authentication)
            //.wrap(middleware::auth::Authentication)
            .service(
                web::scope("/static")
                    .service(web::resource("/{filename:.*}").route(web::get().to(load_file))),
            )
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/static") // doesn't matter if '/api' or not
                            .service(
                                web::resource("/{filename:.*}").route(web::get().to(load_file)),
                            ),
                    )
                    .service(
                        web::resource("/signin")
                            .route(web::post().to(routes::user::signin)),
                    )
                    .service(
                        web::resource("/user/{uid}/token")
                            .route(web::put().to(routes::user::update_token)),
                    )
                    .service(
                        web::resource("/user/{uid}/blacklist")
                            .route(web::get().to(routes::blacklist::get_all))
                            .route(web::post().to(routes::blacklist::add))
                            .route(web::put().to(routes::blacklist::delete)), //deletes
                    )
                    /*.service(
                        web::resource("/user/{uid}/profile")
                            .route(web::post().to_async(routes::user::upload_profile_picture)),
                    )*/
                    .service(
                        web::resource("/user/{uid}")
                            .route(web::get().to(routes::user::get))
                            .route(web::put().to(routes::user::update)),
                    )
                    .service(
                        web::resource("/contacts/{uid}/{country_code}")
                            .route(web::post().to(routes::contacts::create)),
                    )
                    .service(
                        web::resource("/contacts/{uid}")
                            .route(web::get().to(routes::contacts::get_contacts)),
                    )
                    .service(
                        web::resource("/auth/request_code")
                            .route(web::post().to(routes::number_registration::request_code)),
                    )
                    .service(
                        web::resource("/auth/check")
                            .route(web::post().to(routes::number_registration::check)),
                    )
                    .default_service(web::route().to(HttpResponse::NotFound)),
            )
    })
    .keep_alive(None);

    let listener = server.bind(format!("{}:{}", addr, port));

    listener.expect("Cannot bind").run().await
}

async fn load_file(req: actix_web::HttpRequest) -> Result<NamedFile, ServiceError> {
    let path: PathBuf = req
        .match_info()
        .query("filename")
        .parse()
        .map_err(|_err| ServiceError::BadRequest("filename missing".to_string()))?;
    let mut dir = PathBuf::from("static");
    dir.push(path);
    Ok(NamedFile::open(dir).map_err(|err| {
        error!("load_file {:?}", err);
        ServiceError::InternalServerError(InternalServerError::IOError(err.to_string()))
    })?)
}
