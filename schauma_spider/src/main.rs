use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{middleware as actix_middleware, web, App, HttpResponse, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::run_pending_migrations;

use schauma::routes::events::populate_events;

#[allow(dead_code)]
fn set_testing_datasources() -> schauma::datasources::EventDatasourceWrapper {
    schauma::datasources::EventDatasourceWrapper {
        service: Box::new(schauma::datasources::testing::TestingDatasource)
    }
}

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

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

    let connection: &PgConnection = &pool.get().unwrap();
    run_pending_migrations(connection).expect("cannot run pending migrations");

    let server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(set_testing_datasources())
            .wrap(
                Cors::new()
                    .allowed_origin("http://localhost:3000")
                    //.allowed_origin("https://schauma.xyz")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .wrap(actix_middleware::Logger::default())
            .data(web::JsonConfig::default().limit(4048 * 1024))
            .wrap(actix_middleware::Compress::default())
            .service(
                web::scope("/services")
                    .service(
                        web::resource("/create_events/{date}")
                            .route(web::post().to_async(populate_events)),
                    )
                    .default_service(web::route().to(|| HttpResponse::NotFound())),
            )
    })
    .keep_alive(None);

    let listener = server.bind(format!("{}:{}", addr, port));

    listener.expect("Cannot bind").run().unwrap()
}
