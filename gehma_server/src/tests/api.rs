use super::*;
use web_contrib::auth::testing::*;
use web_contrib::auth::AuthenticatorWrapper;

use actix_web::{test, web, App};
use core::models::dto::*;
//use diesel_migrations::run_pending_migrations;
use serde_json::json;

use data_encoding::HEXUPPER;
use ring::digest;

use crate::persistence::blacklist::{MockPersistentBlacklistDao, PersistentBlacklistDao};
use crate::persistence::contact_exists::{
    MockPersistentContactExistsDao, PersistentContactExistsDao,
};
use crate::persistence::user::{MockPersistentUserDao, PersistentUserDao};
use crate::ratelimits::{DefaultRateLimitPolicy, RateLimitWrapper};

use uuid::Uuid;

fn set_testing_auth() -> AuthenticatorWrapper {
    let config = TestingAuthConfiguration {
        id: "test".to_string(),
        auth_token: "test".to_string(),
    };

    AuthenticatorWrapper::new(Box::new(TestingAuthentificator { config: config }))
}

fn set_ratelimits() -> RateLimitWrapper {
    RateLimitWrapper::new(Box::new(DefaultRateLimitPolicy))
}

fn hash(value: impl Into<String>) -> String {
    HEXUPPER.encode(digest::digest(&digest::SHA256, value.into().as_bytes()).as_ref())
}

macro_rules! init_server {
    ($user_dao:ident, $blacklist_dao:ident, $contact_exists_dao:ident) => {
        test::init_service(
            App::new()
                .data(set_testing_auth())
                .data(set_ratelimits())
                .data(Box::new($user_dao) as Box<dyn PersistentUserDao>)
                .data(Box::new($blacklist_dao) as Box<dyn PersistentBlacklistDao>)
                .data(Box::new($contact_exists_dao) as Box<dyn PersistentContactExistsDao>)
                .route(
                    "/api/auth/request_code",
                    web::post().to(crate::routes::auth::request_code),
                )
                .route(
                    "/api/auth/check",
                    web::post().to(crate::routes::auth::check),
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
                    "/api/exists/{uid}/{country_code}",
                    web::post().to(crate::routes::contact_exists::exists),
                ),
        )
    };
}

macro_rules! setup_login_account {
    ($user_dao:ident) => {
        $user_dao //login
            .expect_get_by_id()
            .times(1)
            .returning(|id, _access_token| {
                Ok(UserDto {
                    id: *id,
                    tele_num: "+4366412345678".to_string(),
                    led: false,
                    country_code: "AT".to_string(),
                    description: "".to_string(),
                    changed_at: chrono::Utc::now().naive_local(),
                    profile_picture: "".to_string(),
                    hash_tele_num: hash("+4366412345678".to_string()),
                    xp: 0,
                    client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
                })
            });
    };
}

#[actix_rt::test]
async fn test_create_user() {
    let mut user_dao_mock = MockPersistentUserDao::new();
    let blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let contact_exists_dao_mock = MockPersistentContactExistsDao::new();

    user_dao_mock.expect_create().returning(
        |tele_num, country_code, client_version, _access_token| {
            Ok(UserDto {
                id: Uuid::new_v4(),
                tele_num: tele_num.to_string(),
                led: false,
                country_code: country_code.to_string(),
                description: "".to_string(),
                changed_at: chrono::Utc::now().naive_local(),
                profile_picture: "".to_string(),
                hash_tele_num: hash(tele_num.to_string()),
                xp: 0,
                client_version: client_version.to_string(),
            })
        },
    );

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contact_exists_dao_mock).await;

    let tele_num = "+4366412345678";

    let req = test::TestRequest::post()
        .uri("/api/auth/request_code")
        .set_json(&json! ({
            "tele_num": tele_num,
            "country_code": "AT",
            "client_version": super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
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
            "country_code": "AT",
            "client_version": super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
            "code": "123"
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
    assert_eq!(user.tele_num, tele_num.to_string());
}

#[actix_rt::test]
async fn test_get_user() {
    let mut user_dao_mock = MockPersistentUserDao::new();
    let blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let contact_exists_dao_mock = MockPersistentContactExistsDao::new();

    setup_login_account!(user_dao_mock);

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contact_exists_dao_mock).await;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/user/{}?access_token={}",
            Uuid::new_v4(),
            "TEST".to_string()
        ))
        .to_request();

    let user: UserDto = test::read_response_json(&mut app, req).await;

    assert_eq!(user.tele_num, "+4366412345678");
    assert_eq!(user.country_code, "AT".to_string());
    assert_eq!(user.led, false);
    assert_eq!(user.description, "".to_string());
    assert_eq!(user.xp, 0);
    assert_eq!(
        user.hash_tele_num,
        HEXUPPER.encode(digest::digest(&digest::SHA256, user.tele_num.as_bytes()).as_ref())
    );
}

#[actix_rt::test]
/// This updates the description of an user.
async fn test_update_user() {
    let id = Uuid::new_v4();

    let mut user_dao_mock = MockPersistentUserDao::new();
    let blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let contact_exists_dao_mock = MockPersistentContactExistsDao::new();

    setup_login_account!(user_dao_mock);

    user_dao_mock
        .expect_create_analytics_for_user()
        .times(1)
        .returning(|user| {
            Ok(AnalyticDto {
                id: 1,
                tele_num: user.tele_num.clone(),
                led: user.led,
                description: user.description.clone(),
                created_at: chrono::Utc::now().naive_local(),
            })
        });

    user_dao_mock
        .expect_update_user()
        .times(1)
        .returning(|id, user, current_time| {
            let u = UserDto {
                id: *id,
                tele_num: "+4366412345678".to_string(),
                led: false,
                country_code: "AT".to_string(),
                description: "".to_string(),
                changed_at: chrono::Utc::now().naive_local(),
                profile_picture: "".to_string(),
                hash_tele_num: hash("+4366412345678".to_string()),
                xp: 100,
                client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
            };

            Ok(u.apply_update(user, current_time.naive_local()))
        });

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contact_exists_dao_mock).await;

    let req = test::TestRequest::put()
        .uri(&format!("/api/user/{}?access_token={}", id, "ACCESS_TOKEN"))
        .set_json(&core::models::dto::UpdateUserDto {
            description: "test".to_string(),
            led: true,
            client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
        })
        .to_request();

    /*
    let resp = test::call_service(&mut app, req).await;
    println!("{:?}", resp);
    println!("{:?}", resp.response().error().unwrap());
    */

    let user: UserDto = test::read_response_json(&mut app, req).await;

    assert_eq!(user.tele_num, "+4366412345678".to_string());
    assert_eq!(user.country_code, "AT".to_string());
    assert_eq!(user.led, true);
    assert_eq!(user.xp, 100);
    assert_eq!(user.description, "test".to_string());
}

#[actix_rt::test]
async fn test_update_token_user() {
    let mut user_dao_mock = MockPersistentUserDao::new();
    let blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let contact_exists_dao_mock = MockPersistentContactExistsDao::new();

    setup_login_account!(user_dao_mock);

    user_dao_mock
        .expect_update_token()
        .times(1)
        .returning(|_id, _token| Ok(()));

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contact_exists_dao_mock).await;

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/user/{}/token?access_token={}",
            Uuid::new_v4(),
            "ACCESS_TOKEN"
        ))
        .set_json(&crate::routes::user::UpdateTokenPayload {
            token: "test".to_string(),
        })
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_create_blacklist() {
    env_logger::init();
    std::env::set_var("RUST_LOG", "info,actix_web=info,actix_server=info");

    let mut user_dao_mock = MockPersistentUserDao::new();
    let mut blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let contact_exists_dao_mock = MockPersistentContactExistsDao::new();

    let id = Uuid::new_v4();

    setup_login_account!(user_dao_mock);

    user_dao_mock
        .expect_get_by_hash_tele_num_unsafe()
        .times(1)
        .returning(|hash_tele_num| {
            Ok(UserDto {
                id: Uuid::new_v4(),
                tele_num: "+4365012345678".to_string(),
                led: false,
                country_code: "AT".to_string(),
                description: "".to_string(),
                changed_at: chrono::Utc::now().naive_local(),
                profile_picture: "".to_string(),
                hash_tele_num: hash_tele_num.to_string(),
                xp: 0,
                client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
            })
        });

    blacklist_dao_mock
        .expect_create()
        .times(1)
        .returning(|blocker, blocked| {
            Ok(BlacklistDto {
                id: Uuid::new_v4(),
                created_at: chrono::Utc::now().naive_local(),
                hash_blocker: hash(blocker.to_string()),
                hash_blocked: hash(blocked.to_string()),
            })
        });

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contact_exists_dao_mock).await;

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/user/{}/blacklist?access_token={}",
            id.to_string(),
            "ACCESS"
        ))
        .set_json(&crate::routes::blacklist::PostData {
            hash_blocked: hash("+4365012345678"),
            country_code: "AT".to_string(),
        })
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    //println!("{:?}", resp.response().error().unwrap());

    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_get_all_blacklist() {
    let mut user_dao_mock = MockPersistentUserDao::new();
    let mut blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let contact_exists_dao_mock = MockPersistentContactExistsDao::new();

    setup_login_account!(user_dao_mock);

    blacklist_dao_mock
        .expect_get()
        .times(1)
        .returning(|_blocker| {
            Ok(vec![BlacklistDto {
                id: Uuid::new_v4(),
                created_at: chrono::Utc::now().naive_local(),
                hash_blocker: hash("+4366412345678".to_string()),
                hash_blocked: hash("+4365012345678".to_string()),
            }])
        });

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contact_exists_dao_mock).await;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/user/{}/blacklist?access_token={}",
            Uuid::new_v4(),
            "ACCESS"
        ))
        .to_request();

    let blacklists: Vec<BlacklistDto> = test::read_response_json(&mut app, req).await;

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

#[actix_rt::test]
async fn test_remove_blacklist() {
    let mut user_dao_mock = MockPersistentUserDao::new();
    let mut blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let contact_exists_dao_mock = MockPersistentContactExistsDao::new();

    setup_login_account!(user_dao_mock);

    blacklist_dao_mock
        .expect_delete()
        .times(1)
        .returning(|_blocker, _blocked| {
            assert_eq!(_blocker.to_string(), hash("+4366412345678".to_string()));
            assert_eq!(_blocked.to_string(), hash("+4365012345678".to_string()));

            Ok(())
        });

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contact_exists_dao_mock).await;

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/user/{}/blacklist?access_token={}",
            Uuid::new_v4(),
            "ACCESS"
        ))
        .set_json(&crate::routes::blacklist::PostData {
            hash_blocked: hash("+4365012345678"),
            country_code: "AT".to_string(),
        })
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_contacts() {
    let mut user_dao_mock = MockPersistentUserDao::new();
    let blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let mut contact_exists_dao_mock = MockPersistentContactExistsDao::new();

    setup_login_account!(user_dao_mock);

    contact_exists_dao_mock.expect_get().times(1).returning(
        |_uid, user, phone_number, country_code| {
            assert_eq!(
                phone_number[0].hash_tele_num,
                hash("+4365012345678".to_string())
            );
            assert_eq!(user.country_code, *country_code);

            Ok(vec![WrappedUserDto {
                hash_tele_num: hash("+4365012345678".to_string()),
                name: "Test".to_string(),
                user: Some(UserDto {
                    id: Uuid::new_v4(),
                    tele_num: "+4365012345678".to_string(),
                    led: false,
                    country_code: "AT".to_string(),
                    description: "".to_string(),
                    changed_at: chrono::Utc::now().naive_local(),
                    profile_picture: "".to_string(),
                    hash_tele_num: hash("+4365012345678".to_string()),
                    xp: 0,
                    client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
                }),
            }])
        },
    );

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contact_exists_dao_mock).await;

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/exists/{}/{}?access_token={}",
            Uuid::new_v4(),
            "AT",
            "ACCESS"
        ))
        .set_json(&core::models::dto::PayloadNumbersDto {
            numbers: vec![core::models::dto::PayloadUserDto {
                name: "Test".to_string(),
                hash_tele_num: hash("+4365012345678"),
            }],
        })
        .to_request();

    let users: Vec<WrappedUserDto> = test::read_response_json(&mut app, req).await;

    assert_eq!(users.len(), 1);
    assert_eq!(users.get(0).unwrap().hash_tele_num, hash("+4365012345678"));
    assert_eq!(users.get(0).unwrap().name, "Test".to_string());
    assert_eq!(
        users.get(0).unwrap().user.as_ref().unwrap().tele_num,
        "+4365012345678".to_string()
    );
    assert_eq!(
        users.get(0).unwrap().user.as_ref().unwrap().country_code,
        "AT".to_string()
    );
}

#[actix_rt::test]
async fn test_contacts2() {
    let mut user_dao_mock = MockPersistentUserDao::new();
    let blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let mut contact_exists_dao_mock = MockPersistentContactExistsDao::new();

    setup_login_account!(user_dao_mock);

    contact_exists_dao_mock.expect_get().times(1).returning(
        |_uid, _user, _phone_number, _country_code| {
            Ok(vec![
                WrappedUserDto {
                    hash_tele_num: hash("+4366412345678".to_string()),
                    name: "Test1".to_string(),
                    user: Some(UserDto {
                        id: Uuid::new_v4(),
                        tele_num: "".to_string(),
                        led: false,
                        country_code: "AT".to_string(),
                        description: "".to_string(),
                        changed_at: chrono::Utc::now().naive_local(),
                        profile_picture: "".to_string(),
                        hash_tele_num: hash("+4366412345678".to_string()),
                        xp: 0,
                        client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
                    }),
                },
                WrappedUserDto {
                    hash_tele_num: hash("+4365012345678".to_string()),
                    name: "Test2".to_string(),
                    user: Some(UserDto {
                        id: Uuid::new_v4(),
                        tele_num: "+4365012345678".to_string(),
                        led: false,
                        country_code: "AT".to_string(),
                        description: "".to_string(),
                        changed_at: chrono::Utc::now().naive_local(),
                        profile_picture: "".to_string(),
                        hash_tele_num: hash("+4365012345678".to_string()),
                        xp: 0,
                        client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
                    }),
                },
                WrappedUserDto {
                    hash_tele_num: hash("+43699012345678".to_string()),
                    name: "Test3".to_string(),
                    user: Some(UserDto {
                        id: Uuid::new_v4(),
                        tele_num: "".to_string(),
                        led: false,
                        country_code: "AT".to_string(),
                        description: "".to_string(),
                        changed_at: chrono::Utc::now().naive_local(),
                        profile_picture: "".to_string(),
                        hash_tele_num: hash("+43699012345678".to_string()),
                        xp: 0,
                        client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
                    }),
                },
            ])
        },
    );

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contact_exists_dao_mock).await;

    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/exists/{}/{}?access_token={}",
            Uuid::new_v4(),
            "AT",
            "ACCESS"
        ))
        .set_json(&PayloadNumbersDto {
            numbers: vec![
                PayloadUserDto {
                    name: "Test1".to_string(),
                    hash_tele_num: hash("+4366412345678"),
                },
                PayloadUserDto {
                    name: "Test2".to_string(),
                    hash_tele_num: hash("+4365012345678"),
                },
                PayloadUserDto {
                    name: "Test3".to_string(),
                    hash_tele_num: hash("+43699012345678"),
                },
            ],
        })
        .to_request();

    let users: Vec<WrappedUserDto> = test::read_response_json(&mut app, req).await;
    assert_eq!(users.len(), 3);
}
