use super::*;
use web_contrib::auth::AuthenticatorWrapper;
use web_contrib::auth::testing::*;

use web_contrib::push_notifications::NotificationWrapper;
use web_contrib::push_notifications::testing::*;

use crate::routes::contact_exists::ResponseUser;
use actix_web::{test, web, App};
use core::models::*;
use diesel::pg::PgConnection;
use std::env;

use actix_service::Service;
use diesel_migrations::run_pending_migrations;
use serde_json::json;

use data_encoding::HEXUPPER;
use ring::digest;

fn set_testing_auth() -> AuthenticatorWrapper {
    let config = TestingAuthConfiguration {
        id: "test".to_string(),
        auth_token: "test".to_string(),
    };

    AuthenticatorWrapper::new(Box::new(TestingAuthentificator { config: config }))
}

fn set_notification_service() -> NotificationWrapper {
    let config = TestingNotificationService;

    NotificationWrapper::new(Box::new(config))
}

fn create_user(tele_num: &str) -> User {
    let mut app = test::init_service(
        App::new()
            .data(init_pool())
            .data(set_testing_auth())
            .route(
                "/api/auth/request_code",
                web::post().to_async(crate::routes::auth::request_code),
            )
            .route(
                "/api/auth/check",
                web::post().to_async(crate::routes::auth::check),
            ),
    );

    let r1 = test::TestRequest::post()
        .uri("/api/auth/request_code")
        .set_json(&json! ({
            "tele_num": tele_num,
            "country_code": "AT",
            "client_version": super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
        }))
        .to_request();

    let rr1 = test::block_on(test::run_on(|| app.call(r1))).unwrap();
    assert!(rr1.status().is_success());

    let req = test::TestRequest::post()
        .uri("/api/auth/check")
        .set_json(&json!({
            "tele_num": tele_num,
            "country_code": "AT",
            "client_version": super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
            "code": "123"
        }))
        .to_request();

    let user: User = test::read_response_json(&mut app, req);

    user
}

fn cleanup(mtele_num: &str, conn: &PgConnection) {
    use core::schema::users::dsl::{tele_num, users};

    diesel::delete(users.filter(tele_num.eq(mtele_num)))
        .execute(conn)
        .unwrap();
}

fn init_pool() -> Pool {
    dotenv::dotenv().ok();

    let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let connection: &PgConnection = &pool.get().unwrap();

    run_pending_migrations(connection).expect("cannot run pending migrations");

    pool
}

#[test]
fn test_create_user() {
    let mut app = test::init_service(
        App::new()
            .data(init_pool())
            .data(set_testing_auth())
            .route(
                "/api/auth/request_code",
                web::post().to_async(crate::routes::auth::request_code),
            )
            .route(
                "/api/auth/check",
                web::post().to_async(crate::routes::auth::check),
            ),
    );

    let tele_num = "+4366412345678";

    let _ = test::TestRequest::post()
        .uri("/api/auth/request_code")
        .set_json(&json! ({
            "tele_num": tele_num,
            "country_code": "AT",
            "client_version": super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
        }))
        .to_request();

    let req = test::TestRequest::post()
        .uri("/api/auth/check")
        .set_json(&json!({
            "tele_num": tele_num,
            "country_code": "AT",
            "client_version": super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
            "code": "123"
        }))
        .to_request();

    let user: User = test::read_response_json(&mut app, req);

    assert_eq!(user.tele_num, tele_num.to_string());

    cleanup(&user.tele_num, &init_pool().get().unwrap());
}

#[test]
fn test_sign_in() {
    let user = create_user("+4366412345678");
    assert_eq!(user.tele_num, "+4366412345678".to_string());
    cleanup(&user.tele_num, &init_pool().get().unwrap());
}

#[test]
fn test_get_user() {
    let mut app = test::init_service(App::new().data(init_pool()).data(set_testing_auth()).data(set_notification_service()).route(
        "/api/user/{uid}",
        web::get().to_async(crate::routes::user::get),
    ));

    let user = create_user("+4366412345678");

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/user/{}?access_token={}",
            user.id, user.access_token
        ))
        .to_request();

    /*
    let mut resp = test::block_on(test::run_on(|| app.call(req))).unwrap();
    println!("{:?}", resp);
    println!("{:?}", resp.response().error().unwrap());
    */

    let user: User = test::read_response_json(&mut app, req);

    assert_eq!(user.tele_num, "+4366412345678".to_string());
    assert_eq!(user.country_code, "AT".to_string());
    assert_eq!(user.led, false);
    assert_eq!(user.description, "".to_string());
    assert_eq!(user.hash_tele_num, Some(HEXUPPER.encode(digest::digest(&digest::SHA256, user.tele_num.as_bytes()).as_ref())));

    cleanup(&user.tele_num, &init_pool().get().unwrap());
}

#[test]
/// This updates the description of an user.
fn test_update_user() {
    let mut app = test::init_service(App::new().data(init_pool()).data(set_testing_auth()).data(set_notification_service()).route(
        "/api/user/{uid}",
        web::put().to_async(crate::routes::user::update),
    ));

    let user = create_user("+4366412345678");

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/user/{}?access_token={}",
            user.id, user.access_token
        ))
        .set_json(&crate::routes::user::UpdateUser {
            description: "test".to_string(),
            led: true,
            client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
        })
        .to_request();

    /*
    let mut resp = test::block_on(test::run_on(|| app.call(req))).unwrap();
    println!("{:?}", resp);
    println!("{:?}", resp.response().error().unwrap());
    */

    let user: User = test::read_response_json(&mut app, req);

    assert_eq!(user.tele_num, "+4366412345678".to_string());
    assert_eq!(user.country_code, "AT".to_string());
    assert_eq!(user.led, true);
    assert_eq!(user.description, "test".to_string());

    //FIXME check if notification was sent

    cleanup(&user.tele_num, &init_pool().get().unwrap());
}

#[test]
fn test_update_token_user() {
    let mut app = test::init_service(App::new().data(init_pool()).data(set_testing_auth()).data(set_notification_service()).route(
        "/api/user/{uid}/token",
        web::put().to_async(crate::routes::user::update_token),
    ));

    let user = create_user("+4366412345678");

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/user/{}/token?access_token={}",
            user.id, user.access_token
        ))
        .set_json(&crate::routes::user::UpdateTokenPayload {
            token: "test".to_string(),
        })
        .to_request();

    let resp = test::block_on(test::run_on(|| app.call(req))).unwrap();
    assert!(resp.status().is_success());

    cleanup(&user.tele_num, &init_pool().get().unwrap());
}

#[test]
fn test_create_blacklist() {
    let mut app = test::init_service(App::new().data(init_pool()).data(set_testing_auth()).data(set_notification_service()).route(
        "/api/user/{uid}/blacklist",
        web::post().to_async(crate::routes::blacklist::add),
    ));

    let user = create_user("+4366412345678");
    let user2 = create_user("+4365012345678");

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/user/{}/blacklist?access_token={}",
            user.id, user.access_token
        ))
        .set_json(&crate::routes::blacklist::PostData {
            blocked: "+4365012345678".to_string(),
            country_code: "AT".to_string(),
        })
        .to_request();

    let resp = test::block_on(test::run_on(|| app.call(req))).unwrap();
    assert!(resp.status().is_success());

    cleanup(&user.tele_num, &init_pool().get().unwrap());
    cleanup(&user2.tele_num, &init_pool().get().unwrap());
}

#[test]
fn test_get_all_blacklist() {
    let mut app = test::init_service(
        App::new()
            .data(init_pool())
            .data(set_testing_auth())
            .data(set_notification_service())
            .route(
                "/api/user/{uid}/blacklist",
                web::post().to_async(crate::routes::blacklist::add),
            )
            .route(
                "/api/user/{uid}/blacklist",
                web::get().to_async(crate::routes::blacklist::get_all),
            ),
    );

    let user = create_user("+4366412345678");
    let user2 = create_user("+4365012345678");

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/user/{}/blacklist?access_token={}",
            user.id, user.access_token
        ))
        .set_json(&crate::routes::blacklist::PostData {
            blocked: "+4365012345678".to_string(),
            country_code: "AT".to_string(),
        })
        .to_request();

    let resp = test::block_on(test::run_on(|| app.call(req))).unwrap();
    assert!(resp.status().is_success());

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/user/{}/blacklist?access_token={}",
            user.id, user.access_token
        ))
        .to_request();

    let blacklists: Vec<Blacklist> = test::read_response_json(&mut app, req);

    assert_eq!(blacklists.len(), 1);
    assert_eq!(blacklists.get(0).unwrap().blocker, "+4366412345678");
    assert_eq!(blacklists.get(0).unwrap().blocked, "+4365012345678");

    cleanup(&user.tele_num, &init_pool().get().unwrap());
    cleanup(&user2.tele_num, &init_pool().get().unwrap());
}

#[test]
fn test_remove_blacklist() {
    let mut app = test::init_service(
        App::new()
            .data(init_pool())
            .data(set_testing_auth())
            .data(set_notification_service())
            .route(
                "/api/user/{uid}/blacklist",
                web::post().to_async(crate::routes::blacklist::add),
            )
            .route(
                "/api/user/{uid}/blacklist",
                web::put().to_async(crate::routes::blacklist::delete),
            ),
    );

    let user = create_user("+4366412345678");
    let user2 = create_user("+4365012345678");

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/user/{}/blacklist?access_token={}",
            user.id, user.access_token
        ))
        .set_json(&crate::routes::blacklist::PostData {
            blocked: "+4365012345678".to_string(),
            country_code: "AT".to_string(),
        })
        .to_request();

    let resp = test::block_on(test::run_on(|| app.call(req))).unwrap();
    assert!(resp.status().is_success());

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/user/{}/blacklist?access_token={}",
            user.id, user.access_token
        ))
        .set_json(&crate::routes::blacklist::PostData {
            blocked: "+4365012345678".to_string(),
            country_code: "AT".to_string(),
        })
        .to_request();

    let resp = test::block_on(test::run_on(|| app.call(req))).unwrap();
    assert!(resp.status().is_success());

    cleanup(&user.tele_num, &init_pool().get().unwrap());
    cleanup(&user2.tele_num, &init_pool().get().unwrap());
}

#[test]
fn test_contacts() {
    let mut app = test::init_service(App::new().data(init_pool()).data(set_testing_auth()).data(set_notification_service()).route(
        "/api/exists/{uid}/{country_code}",
        web::post().to_async(crate::routes::contact_exists::exists),
    ));

    let user = create_user("+4366412345678");
    let user2 = create_user("+4365012345678");

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/exists/{}/{}?access_token={}",
            user.id, "AT", user.access_token
        ))
        .set_json(&crate::routes::contact_exists::Payload {
            numbers: vec![crate::routes::contact_exists::PayloadUser {
                name: "Test".to_string(),
                tele_num: "+4365012345678".to_string(), //TODO for #27 required
            }],
        })
        .to_request();

    let users: Vec<ResponseUser> = test::read_response_json(&mut app, req);

    assert_eq!(users.len(), 1);
    assert_eq!(
        users.get(0).unwrap().calculated_tele,
        "+4365012345678".to_string()
    );
    assert_eq!(users.get(0).unwrap().name, "Test".to_string());
    assert_eq!(
        users.get(0).unwrap().user.as_ref().unwrap().tele_num,
        "+4365012345678".to_string()
    );
    assert_eq!(
        users.get(0).unwrap().user.as_ref().unwrap().country_code,
        "AT".to_string()
    );

    cleanup(&user.tele_num, &init_pool().get().unwrap());
    cleanup(&user2.tele_num, &init_pool().get().unwrap());
}

#[test]
fn test_contacts2() {
    let mut app = test::init_service(App::new().data(init_pool()).data(set_testing_auth()).data(set_notification_service()).route(
        "/api/exists/{uid}/{country_code}",
        web::post().to_async(crate::routes::contact_exists::exists),
    ));

    let user = create_user("+4366412345678");
    let user2 = create_user("+4365012345678");

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/exists/{}/{}?access_token={}",
            user.id, "AT", user.access_token
        ))
        .set_json(&crate::routes::contact_exists::Payload {
            numbers: vec![crate::routes::contact_exists::PayloadUser {
                name: "Test".to_string(),
                tele_num: "+4365012345678".to_string(),
            }, crate::routes::contact_exists::PayloadUser {
                name: "Ich".to_string(),
                tele_num: "+4366412345678".to_string(),
            },],
        })
        .to_request();

    let users: Vec<ResponseUser> = test::read_response_json(&mut app, req);

    assert_eq!(users.len(), 2);
    assert_eq!(
        users.get(0).unwrap().calculated_tele,
        "+4365012345678".to_string()
    );
    assert_eq!(users.get(0).unwrap().name, "Test".to_string());
    assert_eq!(
        users.get(0).unwrap().user.as_ref().unwrap().tele_num,
        "+4365012345678".to_string()
    );
    assert_eq!(
        users.get(0).unwrap().user.as_ref().unwrap().country_code,
        "AT".to_string()
    );
    // User 2
    assert_eq!(
        users.get(1).unwrap().calculated_tele,
        "+4366412345678".to_string()
    );
    assert_eq!(users.get(1).unwrap().name, "Ich".to_string());
    assert_eq!(
        users.get(1).unwrap().user.as_ref().unwrap().tele_num,
        "+4366412345678".to_string()
    );
    assert_eq!(
        users.get(1).unwrap().user.as_ref().unwrap().country_code,
        "AT".to_string()
    );

    cleanup(&user.tele_num, &init_pool().get().unwrap());
    cleanup(&user2.tele_num, &init_pool().get().unwrap());
}

#[test]
fn test_empty_contacts() {
    let mut app = test::init_service(App::new().data(init_pool()).data(set_testing_auth()).data(set_notification_service()).route(
        "/api/exists/{uid}/{country_code}",
        web::post().to_async(crate::routes::contact_exists::exists),
    ));

    let user = create_user("+4366412345678");

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/exists/{}/{}?access_token={}",
            user.id, "AT", user.access_token
        ))
        .set_json(&crate::routes::contact_exists::Payload {
            numbers: Vec::new(),
        })
        .to_request();

    let users: Vec<ResponseUser> = test::read_response_json(&mut app, req);

    assert_eq!(users.len(), 0);
    cleanup(&user.tele_num, &init_pool().get().unwrap());
}
