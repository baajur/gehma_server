use super::*;
use crate::auth::AuthenticatorWrapper;
use actix_web::{test, web, App};
use core::models::*;
use diesel::pg::PgConnection;
use std::env;

use actix_service::Service;
use diesel_migrations::run_pending_migrations;

fn set_testing_auth() -> AuthenticatorWrapper {
    use crate::auth::testing::*;

    let config = TestingAuthConfiguration {
        id: "test".to_string(),
        auth_token: "test".to_string(),
    };

    AuthenticatorWrapper::new(Box::new(TestingAuthentificator { config: config }))
}

fn create_user(tele_num: &str) -> User {
    let mut app = test::init_service(App::new().data(init_pool()).data(set_testing_auth()).route(
        "/api/signin",
        web::post().to_async(crate::routes::user::signin),
    ));

    let req = test::TestRequest::post()
        .uri(&format!("/api/signin?firebase_uid={}", "test"))
        .set_json(&crate::routes::user::PostUser {
            tele_num: tele_num.to_string(),
            country_code: "AT".to_string(),
            client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
        })
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
fn test_sign_in() {
    let user = create_user("+4366412345678");
    assert_eq!(user.tele_num, "+4366412345678".to_string());
    cleanup(&user.tele_num, &init_pool().get().unwrap());
}

#[test]
fn test_get_user() {
    let mut app = test::init_service(App::new().data(init_pool()).data(set_testing_auth()).route(
        "/api/user/{uid}",
        web::get().to_async(crate::routes::user::get),
    ));

    let user = create_user("+4366412345678");

    let req = test::TestRequest::get()
        .uri(&format!("/api/user/{}?firebase_uid={}", user.id, "test"))
        .to_request();

    /*
    let resp = test::block_on(app.call(req)).unwrap();
    println!("{:?}", resp);
    println!("{:?}", resp.response().error().unwrap());
    */

    let user: User = test::read_response_json(&mut app, req);

    assert_eq!(user.tele_num, "+4366412345678".to_string());
    assert_eq!(user.country_code, "AT".to_string());
    assert_eq!(user.led, false);
    assert_eq!(user.description, "".to_string());

    cleanup(&user.tele_num, &init_pool().get().unwrap());
}

#[test]
/// This updates the description of an user.
fn test_update_user() {
    let mut app = test::init_service(App::new().data(init_pool()).data(set_testing_auth()).route(
        "/api/user/{uid}",
        web::put().to_async(crate::routes::user::update),
    ));

    let user = create_user("+4366412345678");

    let req = test::TestRequest::put()
        .uri(&format!("/api/user/{}?firebase_uid={}", user.id, "test"))
        .set_json(&crate::routes::user::UpdateUser {
            description: "test".to_string(),
            led: "true".to_string(),
            client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
        })
        .to_request();

    /*
    let resp = test::block_on(app.call(req)).unwrap();
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
    let mut app = test::init_service(App::new().data(init_pool()).data(set_testing_auth()).route(
        "/api/user/{uid}/token",
        web::put().to_async(crate::routes::push_notification::update_token),
    ));

    let user = create_user("+4366412345678");

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/user/{}/token?firebase_uid={}",
            user.id, "test"
        ))
        .set_json(&crate::routes::push_notification::Payload {
            token: "test".to_string(),
        })
        .to_request();

    /*
    let resp = test::block_on(app.call(req)).unwrap();
    println!("{:?}", resp);
    println!("{:?}", resp.response().error().unwrap());
    */

    let user: User = test::read_response_json(&mut app, req);

    assert_eq!(user.tele_num, "+4366412345678".to_string());
    assert_eq!(user.country_code, "AT".to_string());
    assert_eq!(user.firebase_token, Some("test".to_string()));

    cleanup(&user.tele_num, &init_pool().get().unwrap());
}
