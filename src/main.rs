extern crate diesel;
#[macro_use]
extern crate serde_derive;

use crate::auth::AuthenticatorWrapper;
use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::http::header;
use actix_web::{middleware as actix_middleware, web, App, HttpResponse, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::path::PathBuf;

mod utils;
#[macro_use]
mod auth;
pub(crate) mod controllers;
pub(crate) mod queries;
pub(crate) mod routes;

mod middleware;

#[cfg(test)]
mod tests;

pub const ALLOWED_CLIENT_VERSIONS: &[&'static str] = &["0.5.1"];
pub const LIMIT_PUSH_NOTIFICATION_CONTACTS: usize = 128;
pub const ALLOWED_PROFILE_PICTURE_SIZE: usize = 10_000; //in Kilobytes

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

fn get_auth() -> crate::auth::AuthenticatorWrapper {
    use crate::auth::twilio::TwilioAuthenticator;
    use crate::auth::twilio::TwilioConfiguration;

    let project_id = std::env::var("TWILIO_PROJECT_ID").expect("no PROJECT_ID");
    let auth_token = std::env::var("TWILIO_AUTH_TOKEN").expect("no AUTH_TOKEN");
    let sid = std::env::var("TWILIO_ACCOUNT_ID").expect("no ACCOUNT_ID");

    let config = TwilioConfiguration {
        project_id: project_id,
        account_id: sid,
        auth_token: auth_token,
    };

    crate::auth::AuthenticatorWrapper::new(Box::new(TwilioAuthenticator {
        config
    }))
}

fn set_testing_auth() -> AuthenticatorWrapper {
    use crate::auth::testing::*;

    let config = TestingAuthConfiguration {
        id: "test".to_string(),
        auth_token: "test".to_string(),
    };

    AuthenticatorWrapper::new(Box::new(TestingAuthentificator { config: config }))
}

pub(crate) fn main() {
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

    let server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(set_testing_auth())
            .wrap(
                Cors::new()
                    .allowed_origin("http://localhost:3000")
                    .allowed_origin("https://gehma.xyz")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600),
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
                        web::resource("/signin") //must have query string firebase_uid
                            .route(web::post().to_async(routes::user::signin)),
                    )
                    .service(
                        web::resource("/user/{uid}/token").route(
                            web::put().to_async(routes::push_notification::update_token),
                        ),
                    )
                    .service(
                        web::resource("/user/{uid}/blacklist")
                            .route(web::get().to_async(routes::blacklist::get_all))
                            .route(web::post().to_async(routes::blacklist::add))
                            .route(web::put().to_async(routes::blacklist::delete)), //deletes
                    )
                    .service(
                        web::resource("/user/{uid}/profile")
                            .route(web::post().to_async(routes::user::upload_profile_picture)),
                    )
                    .service(
                        web::resource("/user/{uid}")
                            .route(web::get().to_async(routes::user::get))
                            .route(web::put().to_async(routes::user::update)),
                    )
                    .service(
                        web::resource("/exists/{uid}/{country_code}")
                            .route(web::post().to_async(routes::contact_exists::exists)),
                    )
                    .service(
                        web::resource("/auth/request_code")
                            .route(web::post().to_async(routes::auth::request_code)),
                    )
                    .service(
                        web::resource("/auth/check")
                            .route(web::post().to_async(routes::auth::check)),
                    )
                    .default_service(web::route().to(|| HttpResponse::NotFound())),
            )
    })
    .keep_alive(None);

    let listener = server.bind(format!("{}:{}", addr, port));

    listener.expect("Cannot bind").run().unwrap()
}

fn load_file(req: actix_web::HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let mut dir = PathBuf::from("static");
    dir.push(path);
    Ok(NamedFile::open(dir)?)
}
