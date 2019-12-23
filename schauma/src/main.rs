extern crate diesel;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate web_contrib;

use web_contrib::auth::AuthenticatorWrapper;
use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{middleware as actix_middleware, web, App, HttpResponse, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::path::PathBuf;
use diesel_migrations::run_pending_migrations;
use web_contrib::push_notifications::NotificationWrapper;

pub(crate) mod controllers;
pub(crate) mod queries;
pub(crate) mod routes;

//#[cfg(test)]
//mod tests;

pub const ALLOWED_CLIENT_VERSIONS: &[&'static str] = &["0.1"];

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

/*
fn get_auth() -> web_contrib::auth::AuthenticatorWrapper {
    use web_contrib::auth::twilio::TwilioAuthenticator;
    use web_contrib::auth::twilio::TwilioConfiguration;

    let project_id = std::env::var("TWILIO_PROJECT_ID").expect("no PROJECT_ID");
    let auth_token = std::env::var("TWILIO_AUTH_TOKEN").expect("no AUTH_TOKEN");
    let sid = std::env::var("TWILIO_ACCOUNT_ID").expect("no ACCOUNT_ID");

    let config = TwilioConfiguration {
        project_id: project_id,
        account_id: sid,
        auth_token: auth_token,
    };

    web_contrib::auth::AuthenticatorWrapper::new(Box::new(TwilioAuthenticator {
        config
    }))
}

fn set_testing_auth() -> AuthenticatorWrapper {
    use web_contrib::auth::testing::*;

    let config = TestingAuthConfiguration {
        id: "test".to_string(),
        auth_token: "test".to_string(),
    };

    AuthenticatorWrapper::new(Box::new(TestingAuthentificator { config: config }))
}

fn set_testing_notification() -> NotificationWrapper {
    use web_contrib::push_notifications::testing::*;

    NotificationWrapper::new(Box::new(TestingNotificationService))
}

fn get_firebase_notification_service() -> NotificationWrapper {
    use web_contrib::push_notifications::firebase::FirebaseConfiguration;
    use web_contrib::push_notifications::firebase::FirebaseNotificationService;

    let api_token = std::env::var("FCM_TOKEN").expect("No FCM_TOKEN configured");

    let config = FirebaseConfiguration {
        fcm_token: api_token
    };

    web_contrib::push_notifications::NotificationWrapper::new(Box::new(FirebaseNotificationService {
        config
    }))
}
*/

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
 //           .data(set_testing_auth())
 //           .data(get_firebase_notification_service())
            .wrap(
                Cors::new()
                    .allowed_origin("http://localhost:3000")
                    .allowed_origin("https://schauma.xyz")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .wrap(actix_middleware::Logger::default())

            .data(web::JsonConfig::default().limit(4048 * 1024))
            .wrap(actix_middleware::Compress::default())
            .service(
                web::scope("/api")
                    .default_service(web::route().to(|| HttpResponse::NotFound())),
            )
    })
    .keep_alive(None);

    let listener = server.bind(format!("{}:{}", addr, port));

    listener.expect("Cannot bind").run().unwrap()
}
