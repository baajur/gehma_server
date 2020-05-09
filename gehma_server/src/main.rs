#[macro_use]
extern crate serde_derive;
extern crate web_contrib;

use crate::services::number_registration::testing::*;
use crate::services::number_registration::twilio::*;
use crate::services::number_registration::NumberRegistrationService;
use crate::services::push_notifications::firebase::*;
use crate::services::push_notifications::testing::*;
use crate::services::push_notifications::NotificationService;

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::http::header;
use actix_web::{middleware as actix_middleware, web, App, HttpResponse, HttpServer};
use core::errors::ServiceError;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::run_pending_migrations;
use log::error;
use std::path::PathBuf;


pub(crate) mod controllers;
pub(crate) mod persistence;
pub(crate) mod queries;
pub(crate) mod ratelimits; //move to services
pub(crate) mod routes;
pub(crate) mod services;

pub(crate) mod dao_factory;

//mod middleware;

use dao_factory::*;

#[cfg(test)]
mod tests;

pub const ALLOWED_CLIENT_VERSIONS: &[&str] = &["0.5.4"];
pub const LIMIT_PUSH_NOTIFICATION_CONTACTS: usize = 128;
pub const ALLOWED_PROFILE_PICTURE_SIZE: usize = 10_000; //in Kilobytes

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[allow(dead_code)]
fn get_auth() -> NumberRegistrationService {
    let project_id = std::env::var("TWILIO_PROJECT_ID").expect("no PROJECT_ID");
    let auth_token = std::env::var("TWILIO_AUTH_TOKEN").expect("no AUTH_TOKEN");
    let sid = std::env::var("TWILIO_ACCOUNT_ID").expect("no ACCOUNT_ID");

    let config = TwilioConfiguration {
        project_id,
        account_id: sid,
        auth_token,
    };

    Box::new(TwilioAuthenticator { config })
}

#[allow(dead_code)]
fn set_testing_auth() -> NumberRegistrationService {
    let config = TestingAuthConfiguration {
        id: "test".to_string(),
        auth_token: "test".to_string(),
    };

    Box::new(TestingAuthentificator { config })
}

#[allow(dead_code)]
fn set_testing_auth_false() -> NumberRegistrationService {
    let config = TestingAuthConfiguration {
        id: "test".to_string(),
        auth_token: "test".to_string(),
    };

    Box::new(TestingAuthentificatorAlwaysFalse { config })
}

#[allow(dead_code)]
fn set_testing_notification() -> NotificationService {
    Box::new(TestingNotificationService)
}

#[allow(dead_code)]
fn get_ratelimits() -> ratelimits::RateLimitWrapper {
    ratelimits::RateLimitWrapper::new(Box::new(ratelimits::DefaultRateLimitPolicy))
}


#[allow(dead_code)]
fn get_firebase_notification_service() -> NotificationService {
    let api_token = std::env::var("FCM_TOKEN").expect("No FCM_TOKEN configured");

    let config = FirebaseConfiguration {
        fcm_token: api_token,
    };

    Box::new(FirebaseNotificationService { config })
}

#[actix_rt::main]
pub(crate) async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info,actix_web=info,actix_server=info");
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL expected");

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = std::env::var("BINDING_ADDR").unwrap_or_else(|_| "localhost".to_string());

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create a pool");

    let connection: &PgConnection = &pool.get().unwrap();
    run_pending_migrations(connection).expect("cannot run pending migrations");

    let server = HttpServer::new(move || {
        let dao_factory = DaoFactory::new(pool.clone());

        App::new()
            .data(pool.clone())
            //.data(get_auth())
            .data(set_testing_auth())
            .data(get_firebase_notification_service())
            .data(get_ratelimits())
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
                        web::resource("/signin") //must have query string access_token
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
                            .route(web::put().to(routes::user::update)), //token update
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
        ServiceError::InternalServerError
    })?)
}
