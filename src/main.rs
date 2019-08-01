#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer, Responder};
use actix_web::{http::header};
use actix_web::guard;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

mod errors;
mod models;
mod schema;
mod blacklist_handler;
mod exists_handler;
mod user_handler;
mod utils;

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
            .service(web::scope("/test").service(web::resource("/").route(web::get().to(index))))
            .service(
                web::scope("/api")
                    .service(web::resource("/user").route(web::post().to_async(user_handler::add)))
                    .service(
                        web::resource("/user/{uid}/blacklist")
                            .route(web::get().to_async(blacklist_handler::get_all))
                            .route(web::post().to_async(blacklist_handler::add))
                            .route(web::put().to_async(blacklist_handler::delete)), //deletes
                    )
                    .service(
                        web::resource("/user/{uid}")
                            .route(web::get().to_async(user_handler::get))
                            //.route(web::post().to_async(user_handler::add))
                            .route(web::put().to_async(user_handler::update)),
                    )
                    .service(
                        web::resource("/exists/{uid}/{country_code}")
                            //.route(web::post().guard(guard::Header("Content-Type", "application/json")).to_async(exists_handler::get)),
                            .route(web::post().to_async(exists_handler::get)),
                    ),
            )
    })
    .keep_alive(None);

    let listener = match debug.as_str()  {
        "0" => server.bind_ssl(format!("{}:{}", addr, port), builder),
        "1" => server.bind(format!("{}:{}", addr, port)),
        _ => panic!("debug state not defined")
    };

    //.bind_ssl("0.0.0.0:443", builder)
    //.bind("0.0.0.0:3000")
    listener
    .unwrap()
    .run()
    .unwrap()
}

fn index() -> impl Responder {
    format!("Hello")
}
