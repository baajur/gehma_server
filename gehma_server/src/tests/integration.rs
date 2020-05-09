use crate::ratelimits::{DefaultRateLimitPolicy, RateLimitWrapper};
use data_encoding::HEXUPPER;
use ring::digest;
use uuid::Uuid;

use actix_web::{test, web, App};

use crate::persistence::blacklist::PersistentBlacklistDao;
use crate::persistence::contacts::PersistentContactsDao;
use crate::persistence::user::PersistentUserDao;
use crate::services::push_notifications::NotificationService;

use crate::services::number_registration::{
    NumberRegistrationService, NumberRegistrationServiceTrait,
};

use crate::dao_factory::*;

use crate::services::number_registration::testing::*;
use crate::services::push_notifications::testing::TestingNotificationService;

use crate::Pool;
use core::models::dto::*;
use diesel::r2d2::{self, ConnectionManager};

use serde_json::json;

use diesel::prelude::*;
use diesel_migrations::run_pending_migrations;

fn set_testing_auth() -> NumberRegistrationService {
    let config = TestingAuthConfiguration {
        id: "test".to_string(),
        auth_token: "test".to_string(),
    };

    Box::new(TestingAuthentificator { config: config })
}

fn set_testing_notification_service() -> NotificationService {
    Box::new(TestingNotificationService)
}

fn set_ratelimits() -> RateLimitWrapper {
    RateLimitWrapper::new(Box::new(DefaultRateLimitPolicy))
}

fn hash(value: impl Into<String>) -> HashedTeleNum {
    HashedTeleNum(
        HEXUPPER.encode(digest::digest(&digest::SHA256, value.into().as_bytes()).as_ref()),
    )
}

fn get_pool() -> Pool {
    let database_url = "postgres://psql:test@127.0.0.1:10000/gehma";

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create a pool");

    pool
}

fn get_dao_factory(pool: &Pool) -> DaoFactory {
    let dao_factory = DaoFactory::new(pool.clone());
    dao_factory
}

fn setup_database(pool: &Pool) {
    let connection: &PgConnection = &pool.get().unwrap();
    run_pending_migrations(connection).expect("cannot run pending migrations");
}

macro_rules! init_server_integration_test {
    ($pool:expr) => {{
        setup_database($pool);
        test::init_service(
            App::new()
                .data(set_testing_auth() as Box<dyn NumberRegistrationServiceTrait>)
                .data(set_testing_notification_service() as NotificationService)
                .data(set_ratelimits())
                //.data(get_dao_factory($pool).get_user_dao())
                .data(get_dao_factory($pool).get_blacklist_dao())
                .data(get_dao_factory($pool).get_contacts_dao())
                .data(Box::new(get_dao_factory($pool).get_user_dao()) as Box<dyn PersistentUserDao>)
                .data(Box::new(get_dao_factory($pool).get_blacklist_dao())
                    as Box<dyn PersistentBlacklistDao>)
                .data(Box::new(get_dao_factory($pool).get_contacts_dao())
                    as Box<dyn PersistentContactsDao>)
                .route(
                    "/api/auth/request_code",
                    web::post().to(crate::routes::number_registration::request_code),
                )
                .route(
                    "/api/auth/check",
                    web::post().to(crate::routes::number_registration::check),
                )
                .route("/api/user/{uid}", web::get().to(crate::routes::user::get))
                .route(
                    "/api/user/{uid}",
                    web::put().to(crate::routes::user::update),
                )
                .route(
                    "/api/user/{uid}/token",
                    web::put().to(crate::routes::user::update_token),
                )
                .route(
                    "/api/user/{uid}/blacklist",
                    web::post().to(crate::routes::blacklist::add),
                )
                .route(
                    "/api/user/{uid}/blacklist",
                    web::get().to(crate::routes::blacklist::get_all),
                )
                .route(
                    "/api/user/{uid}/blacklist",
                    web::put().to(crate::routes::blacklist::delete),
                )
                .route(
                    "/api/contacts/{uid}/{country_code}",
                    web::post().to(crate::routes::contacts::create),
                )
                .route(
                    "/api/contacts/{uid}",
                    web::get().to(crate::routes::contacts::get_contacts),
                ),
        )
    }};
}

fn cleanup(pool: &Pool) {
    use diesel::sql_query;
    sql_query("DELETE FROM users;")
        .execute(&pool.get().unwrap())
        .unwrap();
}

async fn create_user() -> UserDto {
    let pool = get_pool();
    let mut app = init_server_integration_test!(&pool).await;

    let tele_num = "+4366412345678";
    let country_code = "AT";
    let client_version = super::ALLOWED_CLIENT_VERSIONS[0].to_string();
    let code = "123";

    let req = test::TestRequest::post()
        .uri("/api/auth/request_code")
        .set_json(&json! ({
            "tele_num": tele_num,
            "country_code": country_code,
            "client_version": client_version,
        }))
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    eprintln!("Request_code called");

    let req = test::TestRequest::post()
        .uri("/api/auth/check")
        .set_json(&json!({
            "tele_num": tele_num,
            "country_code": country_code,
            "client_version": client_version,
            "code": code
        }))
        .to_request();

    let user: UserDto = test::read_response_json(&mut app, req).await;

    eprintln!("auth/check called");

    /*
    let resp = test::call_service(&mut app, req).await;

    if !resp.status().is_success() {
        eprintln!("{:?}", resp.response().error().unwrap());
    }
    */

    user
}

macro_rules! get_user {
    ($app:ident, $cmp_user:ident) => {{
        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/user/{}?access_token={}",
                $cmp_user.id.to_string(),
                $cmp_user.access_token.clone().expect("no access token")
            ))
            .to_request();

        let user: UserDto = test::read_response_json(&mut $app, req).await;

        user
    }};
}

#[actix_rt::test]
async fn test_create_user() {
    let pool = get_pool();
    let mut app = init_server_integration_test!(&pool).await;

    cleanup(&pool);

    let tele_num = "+4366412345678";
    let country_code = "AT";
    let client_version = super::ALLOWED_CLIENT_VERSIONS[0].to_string();
    let code = "123";

    let req = test::TestRequest::post()
        .uri("/api/auth/request_code")
        .set_json(&json! ({
            "tele_num": tele_num,
            "country_code": country_code,
            "client_version": client_version,
        }))
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    if !resp.status().is_success() {
        println!("{:?}", resp.response().error().unwrap());
    }

    assert!(resp.status().is_success());

    let req = test::TestRequest::post()
        .uri("/api/auth/check")
        .set_json(&json!({
            "tele_num": tele_num,
            "country_code": country_code,
            "client_version": client_version,
            "code": code
        }))
        .to_request();

    /*
    let resp = test::call_service(&mut app, req).await;

    if !resp.status().is_success() {
        println!("{:?}", resp.response().error().unwrap());
    }

    assert!(resp.status().is_success());
    */

    let user: UserDto = test::read_response_json(&mut app, req).await;

    cleanup(&pool);
    assert_eq!(user.tele_num, tele_num.to_string());
}

#[actix_rt::test]
async fn test_get_user() {
    let pool = get_pool();

    cleanup(&pool);

    let cmp_user = create_user().await;

    let mut app = init_server_integration_test!(&pool).await;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/user/{}?access_token={}",
            cmp_user.id.to_string(),
            cmp_user.access_token.expect("no access token")
        ))
        .to_request();

    let user: UserDto = test::read_response_json(&mut app, req).await;

    /*let resp = test::call_service(&mut app, req).await;

    if !resp.status().is_success() {
        println!("{:?}", resp.response().error().unwrap());
    }*/

    cleanup(&pool);

    assert_eq!(user.tele_num, cmp_user.tele_num);
    assert_eq!(user.country_code, cmp_user.country_code);
    assert_eq!(user.led, cmp_user.led);
    assert_eq!(user.description, cmp_user.description);
    assert_eq!(user.xp, cmp_user.xp);
    assert_eq!(user.hash_tele_num, cmp_user.hash_tele_num);
    assert!(user.access_token.is_none());
}

#[actix_rt::test]
async fn test_update_token_user() {
    let pool = get_pool();

    cleanup(&pool);

    let cmp_user = create_user().await;

    let mut app = init_server_integration_test!(&pool).await;

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/user/{}/token?access_token={}",
            cmp_user.id.to_string(),
            cmp_user.access_token.clone().unwrap(),
        ))
        .set_json(&crate::routes::user::UpdateTokenPayload {
            token: "test".to_string(),
        })
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    let updated_user = get_user!(app, cmp_user);
}
