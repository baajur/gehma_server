#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::http::header;
use actix_web::{middleware, web, App, HttpServer, Responder};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::path::PathBuf;

use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

mod blacklist_handler;
mod errors;
mod exists_handler;
mod models;
//mod push_notifications;
mod schema;
mod user_handler;
mod utils;
mod push_notification_handler;

pub const ALLOWED_CLIENT_VERSIONS: &'static [&'static str] = &["0.1"];
pub const LIMIT_PUSH_NOTIFICATION_CONTACTS: usize = 128;

fn main() {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info,actix_server=info");
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL expected");
    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    let debug = std::env::var("DEBUG").unwrap_or("1".to_string());
    let addr = std::env::var("BINDING_ADDR").unwrap_or("localhost".to_string());

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: models::Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create a pool");

    //let domain: String = std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".into());

    let server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(
                Cors::new()
                    .allowed_origin("http://localhost:3000")
                    .allowed_origin("https://gehma.xyz")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(50_000))
            //.data(web::JsonConfig::default())
            .service(web::resource("/").route(web::get().to(load_index_file)))
            .service(
                web::scope("/static")
                    .service(web::resource("/{filename:.*}").route(web::get().to(load_file))),
            )
            .service(
                web::scope("/api")
                    .service(web::resource("/user").route(web::post().to_async(user_handler::add)))
                    .service(web::resource("/user/{uid}/token").route(web::put().to_async(push_notification_handler::update_token)))
                    /*.service(
                        web::resource("/ws/{uid}")
                            .route(web::get().to_async(push_notifications::ws_route)),
                    )
                    */
                    .service(
                        web::resource("/user/{uid}/blacklist")
                            .route(web::get().to_async(blacklist_handler::get_all))
                            .route(web::post().to_async(blacklist_handler::add))
                            .route(web::put().to_async(blacklist_handler::delete)), //deletes
                    )
                    .service(
                        web::resource("/user/{uid}")
                            .route(web::get().to_async(user_handler::get))
                            .route(web::put().to_async(user_handler::update)),
                    )
                    .service(
                        web::resource("/exists/{uid}/{country_code}")
                            .route(web::post().to_async(exists_handler::exists)),
                    ),
            )
    })
    .keep_alive(None);

    let listener = match debug.as_str() {
        "0" => server.bind_ssl(format!("{}:{}", addr, port), builder),
        "1" => server.bind(format!("{}:{}", addr, port)),
        _ => panic!("debug state not defined"),
    };

    listener.unwrap().run().unwrap()
}

fn index() -> impl Responder {
    format!("Hello")
}

fn load_index_file(_req: actix_web::HttpRequest) -> actix_web::Result<NamedFile> {
    println!("HHEHREHRHEHRE");
    let path: PathBuf = PathBuf::from("static/index.html");
    Ok(NamedFile::open(path)?)
}

fn load_file(req: actix_web::HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let mut dir = PathBuf::from("static");
    dir.push(path);
    Ok(NamedFile::open(dir)?)
}
