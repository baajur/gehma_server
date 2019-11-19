extern crate diesel;
#[macro_use]
extern crate serde_derive;

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_service::Service;
use actix_web::http::header;
use actix_web::{middleware as actix_middleware, web, App, HttpResponse, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::cell::Cell;
use std::path::PathBuf;

mod utils;
#[macro_use]
mod auth;
pub(crate) mod controllers;
pub(crate) mod queries;

mod middleware;

#[cfg(test)]
mod tests;

pub const ALLOWED_CLIENT_VERSIONS: &[&'static str] = &["0.4"];
pub const LIMIT_PUSH_NOTIFICATION_CONTACTS: usize = 128;
pub const ALLOWED_PROFILE_PICTURE_SIZE: usize = 10_000; //in Kilobytes

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub(crate) fn main() {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info,actix_web=info,actix_server=info");
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL expected");

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = std::env::var("BINDING_ADDR").unwrap_or_else(|_| "localhost".to_string());
    let firebase_auth_token = std::env::var("FIREBASE_AUTH_TOKEN").expect("no FIREBASE_AUTH_TOKEN");
    let firebase_project_id = std::env::var("FIREBASE_PROJECT_ID").expect("no FIREBASE_PROJECT_ID");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create a pool");

    let firebase_auth_configuration = auth::FirebaseDatabaseConfiguration {
        firebase_project_id: firebase_project_id,
        firebase_auth_token: firebase_auth_token,
    };

    let server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(firebase_auth_configuration.clone())
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
                web::scope("/static/profile_pictures")
                .service(web::resource("/{filename:.*}").route(web::get().to(load_file))),
            )
            .service(
                web::scope("/api")
                    .service(
                        web::resource("/signin") //must have query string firebase_uid
                            .route(web::post().to_async(controllers::user::signin)),
                    )
                    .service(
                        web::resource("/user/{uid}/token").route(
                            web::put().to_async(controllers::push_notification::update_token),
                        ),
                    )
                    .service(
                        web::resource("/user/{uid}/blacklist")
                            .route(web::get().to_async(controllers::blacklist::get_all))
                            .route(web::post().to_async(controllers::blacklist::add))
                            .route(web::put().to_async(controllers::blacklist::delete)), //deletes
                    )
                    .service(
                        web::resource("/user/{uid}/profile")
                            .route(web::post().to_async(controllers::user::upload_profile_picture)),
                    )
                    .service(
                        web::resource("/user/{uid}")
                            .route(web::get().to_async(controllers::user::get))
                            .route(web::put().to_async(controllers::user::update)),
                    )
                    .service(
                        web::resource("/exists/{uid}/{country_code}")
                            .route(web::post().to_async(controllers::contact_exists::exists)),
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
