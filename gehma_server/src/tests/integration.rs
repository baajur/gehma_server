use super::*;

use actix_web::{test, web, App};

use crate::queries::*;
use crate::services::push_notifications::{
    MockNotificationServiceTrait, NotificationService, NotificationServiceTrait,
};

use crate::services::number_registration::NumberRegistrationServiceTrait;
use core::models::dao::*;

use crate::Pool;
use diesel::query_dsl::RunQueryDsl;
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;

use serde_json::json;

use diesel_migrations::run_pending_migrations;

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

macro_rules! init_data {
    ($pool:expr) => {
        use core::schema::profile_pictures::dsl::profile_pictures;
        use diesel::prelude::*;
        use diesel::sql_query;

        /*
        let connection: &PgConnection = &$pool.get().unwrap();

        sql_query("DELETE FROM profile_pictures;")
            .execute(connection)
            .unwrap();

        diesel::insert_into(profile_pictures)
            .values(&ProfilePictureDao {
                id: 0,
                path: "path".to_string(),
            })
            .on_conflict_do_nothing()
            .execute(connection)
            .unwrap();

        diesel::insert_into(profile_pictures)
            .values(&ProfilePictureDao {
                id: 1,
                path: "path2".to_string(),
            })
            .on_conflict_do_nothing()
            .execute(connection)
            .unwrap();
        */
    };
}

macro_rules! init_server_integration_test {
    ($pool:expr) => {{
        private_init_server_integration_test!(
            $pool,
            set_testing_notification_service() as NotificationService
        )
    }};
}

macro_rules! private_init_server_integration_test {
    ($pool:expr, $notification_service:expr) => {{
        setup_database($pool);
        test::init_service(
            App::new()
                .data(set_testing_auth() as Box<dyn NumberRegistrationServiceTrait>)
                .data($notification_service)
                .data(set_ratelimits())
                .data(get_dao_factory($pool).get_user_dao())
                .data(get_dao_factory($pool).get_blacklist_dao())
                .data(get_dao_factory($pool).get_contacts_dao())
                .data(get_dao_factory($pool).get_profile_pictures_dao())
                .data(get_session_service())
                .wrap(middleware::auth::Authentication)
                .route("/api/signin", web::post().to(crate::routes::user::signin))
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
                    "/api/user/{uid}/profile",
                    web::get().to(crate::routes::profile_pictures::get_all),
                )
                .route(
                    "/api/user/{uid}/profile",
                    web::post().to(crate::routes::user::upload_profile_picture),
                )
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
                )
                .route(
                    "/api/broadcasts/{uid}",
                    web::get().to(routes::broadcast::get_all),
                ),
        )
    }};
}

macro_rules! make_friend {
    ($app:ident, $query_user:ident, $contact_name:expr, $contact_tele_num:expr, $session_token:expr) => {
        let req = test::TestRequest::post()
            .uri(&format!(
                "/api/contacts/{}/{}",
                $query_user.id.to_string(),
                $query_user.country_code.clone(),
            ))
            .header("AUTHORIZATION", $session_token)
            .set_json(&core::models::dto::PayloadNumbersDto {
                numbers: vec![core::models::dto::PayloadUserDto {
                    name: $contact_name.to_string(),
                    hash_tele_num: hash($contact_tele_num),
                }],
            })
            .to_request();

        execute!($app, req);
    };
}

macro_rules! ignore_contact {
    ($app:ident, $query_user:ident, $contact_tele_num:expr, $session_token:expr) => {
        let req = test::TestRequest::post()
            .uri(&format!(
                "/api/user/{}/blacklist",
                $query_user.id.to_string(),
            ))
            .header("AUTHORIZATION", $session_token)
            .set_json(&crate::routes::blacklist::PostData {
                hash_blocked: hash($contact_tele_num).to_string(),
                country_code: "AT".to_string(),
            })
            .to_request();

        execute!($app, req);
    };
}

macro_rules! signin {
    ($app:ident, $query_user:ident) => {{
        let req = test::TestRequest::post()
            .uri(&format!("/api/signin",))
            .header("AUTHORIZATION", $query_user.access_token.clone().unwrap())
            .set_json(&core::models::dto::PostUserDto {
                tele_num: $query_user.tele_num.clone(),
                country_code: $query_user.country_code.clone(),
                client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
            })
            .to_request();

        let user: UserDto = test::read_response_json(&mut $app, req).await;
        user
    }};
}

macro_rules! gehma {
    ($app:ident, $query_user:ident, $descr:expr, $session_token:expr) => {
        let req = test::TestRequest::put()
            .uri(&format!("/api/user/{}", $query_user.id.to_string(),))
            .header("AUTHORIZATION", $session_token)
            .set_json(&core::models::dto::UpdateUserDto {
                description: $descr.to_string(),
                led: true,
                client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
            })
            .to_request();

        execute!($app, req);

        log::debug!("gehma executed");
    };
}

macro_rules! execute {
    ($app: ident, $req:ident) => {{
        let resp = test::call_service(&mut $app, $req).await;
        if !resp.status().is_success() {
            eprintln!("Status {}", resp.status());
            eprintln!(
                "{:?}",
                resp.response().error().expect("Error but no error message")
            );
        }
    }};
}

fn cleanup(pool: &Pool) {
    use diesel::sql_query;

    log::debug!("cleanup");

    sql_query("DELETE FROM users;")
        //.get_results(&pool.get().unwrap())
        .execute(&pool.get().unwrap())
        .unwrap();

    /*
    sql_query("DELETE FROM profile_pictures;")
        .execute(&pool.get().unwrap())
        .unwrap();
    */

    sql_query("DELETE FROM usage_statistics;")
        .execute(&pool.get().unwrap())
        .unwrap();

    sql_query("DELETE FROM analytics;")
        .execute(&pool.get().unwrap())
        .unwrap();

    sql_query("DELETE FROM broadcast;")
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

    log::debug!("User created");

    /*
    let resp = test::call_service(&mut app, req).await;

    if !resp.status().is_success() {
        eprintln!("{:?}", resp.response().error().unwrap());
    }
    */

    user
}

async fn create_user2() -> UserDto {
    let pool = get_pool();
    let mut app = init_server_integration_test!(&pool).await;

    let tele_num = "+4365012345678";
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

    /*
    let resp = test::call_service(&mut app, req).await;

    if !resp.status().is_success() {
        eprintln!("{:?}", resp.response().error().unwrap());
    }
    */

    user
}

async fn create_user3() -> UserDto {
    let pool = get_pool();
    let mut app = init_server_integration_test!(&pool).await;

    let tele_num = "+4366912345678";
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

    /*
    let resp = test::call_service(&mut app, req).await;

    if !resp.status().is_success() {
        eprintln!("{:?}", resp.response().error().unwrap());
    }
    */

    user
}

macro_rules! get_user {
    ($app:ident, $cmp_user:ident, $session_token:expr) => {{
        let req = test::TestRequest::get()
            .uri(&format!("/api/user/{}", $cmp_user.id.to_string()))
            .header("AUTHORIZATION", $session_token)
            .to_request();

        /*
        let resp = test::call_service(&mut $app, req).await;

        if !resp.status().is_success() {
            eprintln!("{:?}", resp.response().error().unwrap());
        }*/

        let user: UserDto = test::read_response_json(&mut $app, req).await;

        user
    }};
}

macro_rules! update_token {
    ($app:ident, $cmp_user:ident, $token:expr, $session_token: expr) => {{
        let req = test::TestRequest::put()
            .uri(&format!("/api/user/{}/token", $cmp_user.id.to_string()))
            .header("AUTHORIZATION", $session_token)
            .set_json(&crate::routes::user::UpdateTokenPayload {
                token: $token.to_string(),
            })
            .to_request();

        let resp = test::call_service(&mut $app, req).await;
        assert!(resp.status().is_success());
    }};
}

macro_rules! change_profile_picture {
    ($app:ident, $cmp_user:ident, $new_id:expr, $session_token: expr) => {{
        let req = test::TestRequest::post()
            .uri(&format!("/api/user/{}/profile", $cmp_user.id.to_string()))
            .header("AUTHORIZATION", $session_token)
            .set_json(&core::models::dto::UpdateProfilePictureDto {
                profile_id: $new_id,
            })
            .to_request();

        let resp = test::call_service(&mut $app, req).await;
        assert!(resp.status().is_success());
    }};
}

macro_rules! get_broadcasts {
    ($app:ident, $cmp_user:ident, $session_token:expr, $mark_seen:expr) => {{
        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/broadcasts/{}?mark_seen={}",
                $cmp_user.id.to_string(),
                $mark_seen.to_string()
            ))
            .header("AUTHORIZATION", $session_token)
            .to_request();

        /*
        let resp = test::call_service(&mut $app, req).await;

        if !resp.status().is_success() {
            eprintln!("{:?}", resp.response().error().unwrap());
        }*/

        let elements: Vec<BroadcastElementDto> = test::read_response_json(&mut $app, req).await;

        elements
    }};
}

#[actix_rt::test]
async fn test_create_user() {
    env_logger::init();
    let pool = get_pool();
    let mut app = init_server_integration_test!(&pool).await;

    cleanup(&pool);

    init_data!(&pool);

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
    assert!(user.session_token.is_none());
}

#[actix_rt::test]
async fn test_get_user() {
    //env_logger::init();
    let pool = get_pool();

    cleanup(&pool);

    init_data!(&pool);

    let cmp_user = create_user().await;

    let mut app = init_server_integration_test!(&pool).await;

    let user_signin = signin!(app, cmp_user);

    let req = test::TestRequest::get()
        .uri(&format!("/api/user/{}", cmp_user.id.to_string()))
        .header("AUTHORIZATION", user_signin.session_token.unwrap())
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
    assert!(user.session_token.is_none());
}

#[actix_rt::test]
async fn test_update_token_user() {
    //env_logger::init();
    let pool = get_pool();

    cleanup(&pool);

    init_data!(&pool);

    let cmp_user = create_user().await;

    let mut app = init_server_integration_test!(&pool).await;

    let user_signin = signin!(app, cmp_user);

    update_token!(
        app,
        cmp_user,
        "token",
        user_signin.session_token.clone().unwrap()
    );

    let updated_user = get_user!(app, cmp_user, user_signin.session_token.unwrap());

    cleanup(&pool);
    assert_eq!("token".to_string(), updated_user.firebase_token.unwrap());
}

#[actix_rt::test]
async fn test_update_description() {
    //env_logger::init();
    let pool = get_pool();

    cleanup(&pool);

    init_data!(&pool);

    let cmp_user = create_user().await;

    let mut app = init_server_integration_test!(&pool).await;
    let user_signin = signin!(app, cmp_user);
    let session_token = user_signin.session_token.unwrap();

    gehma!(app, cmp_user, "updated description", session_token.clone());

    let updated_user = get_user!(app, cmp_user, session_token.clone());

    cleanup(&pool);
    assert!(updated_user.led);
    assert_eq!("updated description".to_string(), updated_user.description);
    assert_eq!(cmp_user.id, updated_user.id);
    assert!(cmp_user.changed_at < updated_user.changed_at);
    //assert_eq!(cmp_user.created_at, updated_user.created_at);
}

#[actix_rt::test]
async fn test_push_notifications() {
    //env_logger::init();
    let pool = get_pool();

    cleanup(&pool);

    init_data!(&pool);

    let cmp_user = create_user().await;
    let cmp_user2 = create_user2().await;
    let cmp_user3 = create_user3().await;

    let mut m = MockNotificationServiceTrait::new();

    m.expect_push().times(1).returning(|contacts| {
        assert_eq!(contacts.len(), 1);
        assert_eq!("Second".to_string(), contacts.get(0).unwrap().0);
        assert_eq!("token2".to_string(), contacts.get(0).unwrap().1);
        Ok(())
    });

    let mut app = private_init_server_integration_test!(
        &pool,
        Box::new(m) as Box<dyn NotificationServiceTrait>
    )
    .await;

    let user_signin = signin!(app, cmp_user);
    let user_signin2 = signin!(app, cmp_user2);
    let user_signin3 = signin!(app, cmp_user3);

    let session_token = user_signin.session_token.unwrap();
    let session_token2 = user_signin2.session_token.unwrap();
    let session_token3 = user_signin3.session_token.unwrap();

    update_token!(app, cmp_user, "token1", session_token.clone());
    update_token!(app, cmp_user2, "token2", session_token2.clone());
    update_token!(app, cmp_user3, "token", session_token3.clone());

    make_friend!(
        app,
        cmp_user,
        "First",
        cmp_user2.tele_num,
        session_token.clone()
    );
    make_friend!(
        app,
        cmp_user2,
        "Second",
        cmp_user.tele_num,
        session_token2.clone()
    );

    gehma!(app, cmp_user, "updated description", session_token.clone());

    let user = get_user!(app, cmp_user, session_token);

    assert_eq!(cmp_user.id, user.id);
    assert_eq!("updated description".to_string(), user.description);
}

#[actix_rt::test]
async fn test_push_notifications_one_friendship() {
    //env_logger::init();
    let pool = get_pool();

    cleanup(&pool);

    init_data!(&pool);

    let cmp_user = create_user().await;
    let cmp_user2 = create_user2().await;
    let cmp_user3 = create_user3().await;

    let mut m = MockNotificationServiceTrait::new();

    m.expect_push().times(1).returning(|contacts| {
        assert_eq!(contacts.len(), 0);
        Ok(())
    });

    let mut app = private_init_server_integration_test!(
        &pool,
        Box::new(m) as Box<dyn NotificationServiceTrait>
    )
    .await;

    let user_signin = signin!(app, cmp_user);
    let user_signin2 = signin!(app, cmp_user2);
    let user_signin3 = signin!(app, cmp_user3);

    let session_token = user_signin.session_token.unwrap();
    let session_token2 = user_signin2.session_token.unwrap();
    let session_token3 = user_signin3.session_token.unwrap();

    update_token!(app, cmp_user, "token1", session_token.clone());
    update_token!(app, cmp_user2, "token2", session_token2.clone());
    update_token!(app, cmp_user3, "token3", session_token3.clone());

    //only one friendship
    make_friend!(
        app,
        cmp_user,
        "First",
        cmp_user2.tele_num,
        session_token.clone()
    );

    gehma!(app, cmp_user, "updated description", session_token.clone());

    let user = get_user!(app, cmp_user, session_token.clone());

    assert_eq!(cmp_user.id, user.id);
    assert_eq!("updated description".to_string(), user.description);
}

#[actix_rt::test]
async fn test_push_notifications_blacklist1() {
    //env_logger::init();
    let pool = get_pool();

    cleanup(&pool);

    init_data!(&pool);

    let cmp_user = create_user().await;
    let cmp_user2 = create_user2().await;
    let cmp_user3 = create_user3().await;

    let mut m = MockNotificationServiceTrait::new();

    m.expect_push().times(1).returning(|contacts| {
        assert_eq!(contacts.len(), 1);
        assert_eq!("Third2".to_string(), contacts.get(0).unwrap().0);
        assert_eq!("token3".to_string(), contacts.get(0).unwrap().1);
        Ok(())
    });

    let mut app = private_init_server_integration_test!(
        &pool,
        Box::new(m) as Box<dyn NotificationServiceTrait>
    )
    .await;

    let user_signin = signin!(app, cmp_user);
    let user_signin2 = signin!(app, cmp_user2);
    let user_signin3 = signin!(app, cmp_user3);

    let session_token = user_signin.session_token.unwrap();
    let session_token2 = user_signin2.session_token.unwrap();
    let session_token3 = user_signin3.session_token.unwrap();

    update_token!(app, cmp_user, "token1", session_token.clone());
    update_token!(app, cmp_user2, "token2", session_token2.clone());
    update_token!(app, cmp_user3, "token3", session_token3.clone());

    make_friend!(
        app,
        cmp_user,
        "First",
        cmp_user2.tele_num.clone(),
        session_token.clone()
    );
    make_friend!(
        app,
        cmp_user,
        "Third",
        cmp_user3.tele_num,
        session_token.clone()
    );
    make_friend!(
        app,
        cmp_user2,
        "Second",
        cmp_user.tele_num.clone(),
        session_token2.clone()
    );
    make_friend!(
        app,
        cmp_user3,
        "Third2",
        cmp_user.tele_num.clone(),
        session_token3.clone()
    );

    ignore_contact!(app, cmp_user, cmp_user2.tele_num, session_token.clone());

    gehma!(app, cmp_user, "updated description", session_token.clone());

    let user = get_user!(app, cmp_user, session_token.clone());

    assert_eq!(cmp_user.id, user.id);
    assert_eq!("updated description".to_string(), user.description);
}

#[actix_rt::test]
async fn test_push_notifications_blacklist2() {
    //env_logger::init();
    let pool = get_pool();

    cleanup(&pool);

    init_data!(&pool);

    let cmp_user = create_user().await;
    let cmp_user2 = create_user2().await;
    let cmp_user3 = create_user3().await;

    let mut m = MockNotificationServiceTrait::new();

    m.expect_push().times(1).returning(|contacts| {
        assert_eq!(contacts.len(), 1);
        assert_eq!("Third2".to_string(), contacts.get(0).unwrap().0);
        assert_eq!("token3".to_string(), contacts.get(0).unwrap().1);
        Ok(())
    });

    let mut app = private_init_server_integration_test!(
        &pool,
        Box::new(m) as Box<dyn NotificationServiceTrait>
    )
    .await;

    let user_signin = signin!(app, cmp_user);
    let user_signin2 = signin!(app, cmp_user2);
    let user_signin3 = signin!(app, cmp_user3);

    let session_token = user_signin.session_token.unwrap();
    let session_token2 = user_signin2.session_token.unwrap();
    let session_token3 = user_signin3.session_token.unwrap();

    update_token!(app, cmp_user, "token1", session_token.clone());
    update_token!(app, cmp_user2, "token2", session_token2.clone());
    update_token!(app, cmp_user3, "token3", session_token3.clone());

    make_friend!(
        app,
        cmp_user,
        "First",
        cmp_user2.tele_num.clone(),
        session_token.clone()
    );
    make_friend!(
        app,
        cmp_user2,
        "Second",
        cmp_user.tele_num.clone(),
        session_token2.clone()
    );
    make_friend!(
        app,
        cmp_user,
        "Third",
        cmp_user3.tele_num.clone(),
        session_token.clone()
    );
    make_friend!(
        app,
        cmp_user3,
        "Third2",
        cmp_user.tele_num.clone(),
        session_token3.clone()
    );

    ignore_contact!(app, cmp_user2, cmp_user.tele_num, session_token2.clone()); //REVERSED HERE

    gehma!(app, cmp_user, "updated description", session_token.clone());

    let user = get_user!(app, cmp_user, session_token.clone());

    assert_eq!(cmp_user.id, user.id);
    assert_eq!("updated description".to_string(), user.description);
}

#[actix_rt::test]
async fn test_create_blacklist() {
    let pool = get_pool();

    cleanup(&pool);

    init_data!(&pool);

    let cmp_user = create_user().await;
    let _cmp_user2 = create_user2().await;

    let mut app = init_server_integration_test!(&pool).await;

    let user_signin = signin!(app, cmp_user);
    let user_signin2 = signin!(app, _cmp_user2);

    let session_token = user_signin.session_token.unwrap();
    let session_token2 = user_signin2.session_token.unwrap();

    ignore_contact!(app, cmp_user, "+4365012345678", session_token);

    cleanup(&pool);
}

#[actix_rt::test]
async fn test_get_all_blacklists() {
    let pool = get_pool();

    cleanup(&pool);

    init_data!(&pool);

    let cmp_user = create_user().await;
    let _cmp_user2 = create_user2().await;

    let mut app = init_server_integration_test!(&pool).await;

    let user_signin = signin!(app, cmp_user);
    let user_signin2 = signin!(app, _cmp_user2);

    let session_token = user_signin.session_token.unwrap();
    let session_token2 = user_signin2.session_token.unwrap();

    // Creating blacklist
    ignore_contact!(app, cmp_user, "+4365012345678", session_token.clone());

    // Get
    let req = test::TestRequest::get()
        .uri(&format!("/api/user/{}/blacklist", cmp_user.id,))
        .header("AUTHORIZATION", session_token)
        .to_request();

    /*
    let resp = test::call_service(&mut app, req).await;

    if !resp.status().is_success() {
        println!("{:?}", resp.response().error().unwrap());
    }*/

    let blacklists: Vec<BlacklistDto> = test::read_response_json(&mut app, req).await;

    cleanup(&pool);

    assert_eq!(blacklists.len(), 1);
    assert_eq!(
        blacklists.get(0).unwrap().hash_blocker,
        hash("+4366412345678")
    );
    assert_eq!(
        blacklists.get(0).unwrap().hash_blocked,
        hash("+4365012345678")
    );
}

/// This test checks if a user (A) blocks an user (B),
/// user (B) `blocked` is true if (A) queries.
#[actix_rt::test]
async fn test_see_if_blocked_perspective_creator() {
    //env_logger::init();
    let pool = get_pool();

    cleanup(&pool);

    init_data!(&pool);

    let cmp_user = create_user().await;
    let _cmp_user2 = create_user2().await;

    let mut app = init_server_integration_test!(&pool).await;

    let user_signin = signin!(app, cmp_user);
    let user_signin2 = signin!(app, _cmp_user2);

    let session_token = user_signin.session_token.unwrap();
    let session_token2 = user_signin2.session_token.unwrap();

    // Creating contact
    make_friend!(
        app,
        cmp_user,
        "test contact",
        "+4365012345678",
        session_token.clone()
    );

    // Creating blacklist
    ignore_contact!(app, cmp_user, "+4365012345678", session_token.clone());

    // Get all blocked
    let req = test::TestRequest::get()
        .uri(&format!("/api/contacts/{}", cmp_user.id,))
        .header("AUTHORIZATION", session_token.clone())
        .to_request();

    let contacts: Vec<ContactDto> = test::read_response_json(&mut app, req).await;

    cleanup(&pool);

    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts.get(0).unwrap().name, "test contact".to_string());
    assert_eq!(
        contacts.get(0).unwrap().user.hash_tele_num,
        hash("+4365012345678")
    );
    assert!(contacts.get(0).unwrap().blocked);
}

/// This test checks if a user (A) blocks an user (B),
/// user (A) `blocked` is true if (B) queries.
#[actix_rt::test]
async fn test_see_if_blocked_perspective_blocked() {
    //env_logger::init();
    let pool = get_pool();

    cleanup(&pool);

    init_data!(&pool);

    let _cmp_user = create_user().await;
    let cmp_user2 = create_user2().await;

    let mut app = init_server_integration_test!(&pool).await;

    let user_signin = signin!(app, _cmp_user);
    let user_signin2 = signin!(app, cmp_user2);

    let session_token = user_signin.session_token.unwrap();
    let session_token2 = user_signin2.session_token.unwrap();

    // Creating contact
    make_friend!(
        app,
        cmp_user2,
        "test contact",
        "+4366412345678",
        session_token2.clone()
    );

    // Creating blacklist
    ignore_contact!(app, cmp_user2, "+4366412345678", session_token2.clone());

    // Get all blocked
    let req = test::TestRequest::get()
        .uri(&format!("/api/contacts/{}", cmp_user2.id,))
        .header("AUTHORIZATION", session_token2.clone())
        .to_request();

    let contacts: Vec<ContactDto> = test::read_response_json(&mut app, req).await;

    cleanup(&pool);

    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts.get(0).unwrap().name, "test contact".to_string());
    assert_eq!(
        contacts.get(0).unwrap().user.hash_tele_num,
        hash("+4366412345678")
    );
    assert!(contacts.get(0).unwrap().blocked);
}

#[actix_rt::test]
async fn test_update_profile_picture() {
    //env_logger::init();
    let pool = get_pool();

    cleanup(&pool);

    init_data!(&pool);

    let cmp_user = create_user().await;

    let mut app = init_server_integration_test!(&pool).await;

    let user_signin = signin!(app, cmp_user);

    change_profile_picture!(app, cmp_user, 1, user_signin.session_token.clone().unwrap());

    let updated_user = get_user!(app, cmp_user, user_signin.session_token.unwrap());

    cleanup(&pool);
    assert_eq!("ghost.png".to_string(), updated_user.profile_picture);
}

#[actix_rt::test]
async fn test_get_all_profile_pictures() {
    //env_logger::init();
    let pool = get_pool();

    cleanup(&pool);

    init_data!(&pool);

    let cmp_user = create_user().await;

    let mut app = init_server_integration_test!(&pool).await;

    let user_signin = signin!(app, cmp_user);

    let req = test::TestRequest::get()
        .uri(&format!("/api/user/{}/profile", cmp_user.id.to_string()))
        .header("AUTHORIZATION", user_signin.session_token.unwrap())
        .to_request();

    /*
    let resp = test::call_service(&mut $app, req).await;

    if !resp.status().is_success() {
        eprintln!("{:?}", resp.response().error().unwrap());
    }*/

    let pictures: Vec<ProfilePictureDto> = test::read_response_json(&mut app, req).await;

    cleanup(&pool);
    assert_eq!(2, pictures.len());
}

#[actix_rt::test]
async fn test_get_broadcasts() {
    env_logger::init();
    let pool = get_pool();

    cleanup(&pool);

    init_data!(&pool);

    let cmp_user = create_user().await;
    let cmp_user2 = create_user2().await;

    let mut app = init_server_integration_test!(&pool).await;

    let user_signin = signin!(app, cmp_user);
    let user_signin2 = signin!(app, cmp_user2);

    let session_token = user_signin.session_token.unwrap();
    let session_token2 = user_signin2.session_token.unwrap();

    // Creating contact
    make_friend!(
        app,
        cmp_user2,
        "test contact",
        "+4366412345678",
        session_token2.clone()
    );
    make_friend!(
        app,
        cmp_user,
        "test contact",
        "+4365012345678",
        session_token.clone()
    );

    update_token!(app, cmp_user, "token", session_token.clone());

    update_token!(app, cmp_user2, "token", session_token2.clone());

    gehma!(app, cmp_user, "updated description", session_token.clone());

    //let updated_user = get_user!(app, cmp_user, session_token.clone());
    let broadcasts = get_broadcasts!(app, cmp_user2, session_token2.clone(), true);

    assert_eq!(broadcasts.len(), 1);
    assert_eq!(
        broadcasts.get(0).unwrap().text,
        "updated description".to_string()
    );
    assert_eq!(
        broadcasts.get(0).unwrap().originator_user.user.tele_num,
        "+4366412345678".to_string()
    );

    // Now, there should be gone, because seen
    let broadcasts = get_broadcasts!(app, cmp_user2, session_token2.clone(), false);

    cleanup(&pool);

    assert_eq!(broadcasts.len(), 0);
}

#[actix_rt::test]
async fn test_get_broadcasts_not_cleared() {
    //env_logger::init();
    let pool = get_pool();

    cleanup(&pool);

    init_data!(&pool);

    let cmp_user = create_user().await;
    let cmp_user2 = create_user2().await;

    let mut app = init_server_integration_test!(&pool).await;

    let user_signin = signin!(app, cmp_user);
    let user_signin2 = signin!(app, cmp_user2);

    let session_token = user_signin.session_token.unwrap();
    let session_token2 = user_signin2.session_token.unwrap();

    // Creating contact
    make_friend!(
        app,
        cmp_user2,
        "test contact",
        "+4366412345678",
        session_token2.clone()
    );
    make_friend!(
        app,
        cmp_user,
        "test contact",
        "+4365012345678",
        session_token.clone()
    );

    update_token!(app, cmp_user, "token", session_token.clone());

    update_token!(app, cmp_user2, "token", session_token2.clone());

    gehma!(app, cmp_user, "updated description", session_token.clone());

    let broadcasts = get_broadcasts!(app, cmp_user2, session_token2.clone(), false); // false changed

    assert_eq!(broadcasts.len(), 1);
    assert_eq!(
        broadcasts.get(0).unwrap().text,
        "updated description".to_string()
    );

    // Now, there should be gone, because seen
    let broadcasts = get_broadcasts!(app, cmp_user2, session_token2.clone(), false);

    cleanup(&pool);

    assert_eq!(broadcasts.len(), 1);
}
