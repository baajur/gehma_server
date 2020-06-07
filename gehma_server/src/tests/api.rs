use super::*;

use actix_web::{test, web, App};
use serde_json::json;

use crate::queries::*;
use core::models::dao::*;
use uuid::Uuid;

use lazy_static::lazy_static;

lazy_static! {
    static ref USER: UserDao = UserDao {
        id: Uuid::new_v4(),
        tele_num: "+4366412345678".to_string(),
        led: false,
        country_code: "AT".to_string(),
        description: "".to_string(),
        changed_at: chrono::Utc::now().naive_local(),
        created_at: chrono::Utc::now().naive_local(),
        profile_picture: Some(0),
        hash_tele_num: hash("+4366412345678".to_string()),
        xp: 0,
        client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
        access_token: "".to_string(),
        firebase_token: None,
    };
}

macro_rules! init_server {
    ($user_dao:ident, $blacklist_dao:ident, $contact_exists_dao:ident) => {
        test::init_service(
            App::new()
                .data(set_testing_auth() as Box<dyn NumberRegistrationServiceTrait>)
                .data(set_testing_notification_service() as NotificationService)
                .data(set_ratelimits())
                .data(get_session_service())
                .data(Box::new($user_dao) as Box<dyn PersistentUserDao>)
                .data(Box::new($blacklist_dao) as Box<dyn PersistentBlacklistDao>)
                .data(Box::new($contact_exists_dao) as Box<dyn PersistentContactsDao>)
                .data(web::JsonConfig::default().limit(4048 * 1024))
                .wrap(actix_middleware::Compress::default())
                //.wrap(middleware::auth::Authentication)
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
                    web::post().to(routes::contacts::create),
                )
                .route(
                    "/api/contacts/{uid}",
                    web::get().to(routes::contacts::get_contacts),
                ),
        )
    };
}

macro_rules! setup_login_account {
    ($user_dao:ident) => {
        $user_dao //login
            .expect_get_by_id()
            .times(1)
            .returning(|_id| Ok(USER.clone()));
    };
}

#[actix_rt::test]
async fn test_create_user() {
    let mut user_dao_mock = MockPersistentUserDao::new();
    let blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let contact_exists_dao_mock = MockPersistentContactsDao::new();

    // Create a new user, because there is no
    user_dao_mock
        .expect_get_by_tele_num()
        .returning(|_| Err(ServiceError::ResourceDoesNotExist));

    user_dao_mock
        .expect_get_profile_picture()
        .returning(|_| Ok("path".to_string()));

    user_dao_mock.expect_create().returning(
        |tele_num, country_code, client_version, _access_token| {
            Ok(UserDao {
                id: Uuid::new_v4(),
                tele_num: tele_num.to_string(),
                led: false,
                country_code: country_code.to_string(),
                description: "".to_string(),
                changed_at: chrono::Utc::now().naive_local(),
                created_at: chrono::Utc::now().naive_local(),
                profile_picture: Some(0),
                hash_tele_num: hash(tele_num.to_string()),
                xp: 0,
                client_version: client_version.to_string(),
                access_token: "".to_string(),
                firebase_token: None,
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
    let contacts_dao_mock = MockPersistentContactsDao::new();

    setup_login_account!(user_dao_mock);

    user_dao_mock
        .expect_get_profile_picture()
        .returning(|_| Ok("path".to_string()));

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contacts_dao_mock).await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/user/{}", Uuid::new_v4(),))
        .to_request();

    
    let user: UserDto = test::read_response_json(&mut app, req).await;

    assert_eq!(user.tele_num, "+4366412345678");
    assert_eq!(user.country_code, "AT".to_string());
    assert_eq!(user.led, false);
    assert_eq!(user.description, "".to_string());
    assert_eq!(user.xp, 0);
    assert_eq!(user.hash_tele_num, hash(user.tele_num));
}

#[actix_rt::test]
async fn test_get_user_with_invalid_id() {
    let user_dao_mock = MockPersistentUserDao::new();
    let blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let contacts_dao_mock = MockPersistentContactsDao::new();

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contacts_dao_mock).await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/user/{}", "WRONGUID",))
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert_eq!(400, resp.status());
    assert_eq!(
        "BadRequest: Invalid UUID",
        resp.response().error().unwrap().to_string()
    );
}

#[actix_rt::test]
async fn test_get_user_with_invalid_login() {
    let mut user_dao_mock = MockPersistentUserDao::new();
    let blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let contacts_dao_mock = MockPersistentContactsDao::new();

    user_dao_mock //login
        .expect_get_by_id()
        .times(1)
        .returning(|_id| Err(ServiceError::Unauthorized));

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contacts_dao_mock).await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/user/{}", Uuid::new_v4(),))
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert_eq!(401, resp.status());
    assert_eq!("Unauthorized", resp.response().error().unwrap().to_string());
}

#[actix_rt::test]
/// This updates the description of an user.
async fn test_update_user() {
    let id = Uuid::new_v4();

    let mut user_dao_mock = MockPersistentUserDao::new();
    let blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let contacts_dao_mock = MockPersistentContactsDao::new();

    setup_login_account!(user_dao_mock);

    user_dao_mock
        .expect_create_analytics_for_user()
        .times(1)
        .returning(|user| {
            Ok(AnalyticDao {
                id: 1,
                tele_num: user.tele_num.clone(),
                led: user.led,
                description: user.description.clone(),
                created_at: chrono::Utc::now().naive_local(),
            })
        });

    user_dao_mock
        .expect_get_profile_picture()
        .returning(|_| Ok("path".to_string()));

    user_dao_mock
        .expect_update_user()
        .times(1)
        .returning(|_id, user, current_time| {
            let u = USER.clone();
            Ok((u.apply_update(user, current_time.naive_local()), vec![]))
        });

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contacts_dao_mock).await;

    let req = test::TestRequest::put()
        .uri(&format!("/api/user/{}", id))
        .set_json(&core::models::dto::UpdateUserDto {
            description: "test".to_string(),
            led: true,
            client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
        })
        .to_request();


    /*
    let resp = test::call_service(&mut app, req).await;
    println!("{:?}", resp);
    //println!("{:?}", resp.response().error().unwrap());
    */

    let user: UserDto = test::read_response_json(&mut app, req).await;

    assert_eq!(user.tele_num, "+4366412345678".to_string());
    assert_eq!(user.country_code, "AT".to_string());
    assert_eq!(user.led, true);
    assert_eq!(user.xp, 0);
    assert_eq!(user.description, "test".to_string());
}

#[actix_rt::test]
async fn test_update_token_user() {
    let mut user_dao_mock = MockPersistentUserDao::new();
    let blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let contacts_dao_mock = MockPersistentContactsDao::new();

    setup_login_account!(user_dao_mock);

    user_dao_mock
        .expect_update_token()
        .times(1)
        .returning(|_id, _token| Ok(()));

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contacts_dao_mock).await;

    let req = test::TestRequest::put()
        .uri(&format!("/api/user/{}/token", Uuid::new_v4(),))
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
    let contacts_dao_mock = MockPersistentContactsDao::new();

    let id = Uuid::new_v4();

    setup_login_account!(user_dao_mock);

    user_dao_mock
        .expect_get_by_hash_tele_num_unsafe()
        .times(1)
        .returning(|_hash_tele_num| Ok(USER.clone()));

    blacklist_dao_mock
        .expect_create()
        .times(1)
        .returning(|blocker, blocked| {
            Ok(BlacklistDao {
                id: Uuid::new_v4(),
                created_at: chrono::Utc::now().naive_local(),
                hash_blocker: hash(blocker.to_string()),
                hash_blocked: hash(blocked.to_string()),
            })
        });

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contacts_dao_mock).await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/user/{}/blacklist", id.to_string(),))
        .set_json(&crate::routes::blacklist::PostData {
            hash_blocked: hash("+4365012345678").to_string(),
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
    let contacts_dao_mock = MockPersistentContactsDao::new();

    setup_login_account!(user_dao_mock);

    blacklist_dao_mock
        .expect_get()
        .times(1)
        .returning(|_blocker| {
            Ok(vec![BlacklistDao {
                id: Uuid::new_v4(),
                created_at: chrono::Utc::now().naive_local(),
                hash_blocker: hash("+4366412345678".to_string()),
                hash_blocked: hash("+4365012345678".to_string()),
            }])
        });

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contacts_dao_mock).await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/user/{}/blacklist", Uuid::new_v4(),))
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
    let contacts_dao_mock = MockPersistentContactsDao::new();

    setup_login_account!(user_dao_mock);

    blacklist_dao_mock
        .expect_delete()
        .times(1)
        .returning(|_blocker, _blocked| {
            assert_eq!(*_blocker, hash("+4366412345678".to_string()));
            assert_eq!(*_blocked, hash("+4365012345678".to_string()));

            Ok(())
        });

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contacts_dao_mock).await;

    let req = test::TestRequest::put()
        .uri(&format!("/api/user/{}/blacklist", Uuid::new_v4(),))
        .set_json(&crate::routes::blacklist::PostData {
            hash_blocked: hash("+4365012345678").to_string(),
            country_code: "AT".to_string(),
        })
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_contacts() {
    let mut user_dao_mock = MockPersistentUserDao::new();
    let mut blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let mut contacts_dao_mock = MockPersistentContactsDao::new();

    user_dao_mock
        .expect_get_by_id()
        .times(2)
        .returning(|_id| Ok(USER.clone()));

    contacts_dao_mock
        .expect_create()
        .times(1)
        .returning(|_uid, _user, _phone_number| Ok(()));

    blacklist_dao_mock
        .expect_get()
        .times(3)
        .returning(|_| Ok(vec![]));

    contacts_dao_mock
        .expect_get_contacts()
        .times(1)
        .returning(|_user, _| {
            let mut u = USER.clone();
            u.led = true;
            u.description = "test".to_string();
            Ok(vec![ContactDto {
                blocked: false,
                name: "Test".to_string(),
                user: u.into("path".to_string()),
            }])
        });

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contacts_dao_mock).await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/contacts/{}/{}", Uuid::new_v4(), "AT",))
        .set_json(&PayloadNumbersDto {
            numbers: vec![PayloadUserDto {
                name: "Test".to_string(),
                hash_tele_num: hash("+4366412345678"),
            }],
        })
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get()
        .uri(&format!("/api/contacts/{}", Uuid::new_v4(),))
        .to_request();

    //let resp = test::call_service(&mut app, req).await;
    //assert!(resp.status().is_success());

    let users: Vec<ContactDto> = test::read_response_json(&mut app, req).await;

    assert!(users.len() > 0);
    assert_eq!(true, users.get(0).unwrap().user.led);
    assert_eq!("test", &users.get(0).unwrap().user.description);
}

#[actix_rt::test]
async fn test_contacts_with_blacklist_1() {
    let mut user_dao_mock = MockPersistentUserDao::new();
    let mut blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let mut contacts_dao_mock = MockPersistentContactsDao::new();

    user_dao_mock
        .expect_get_by_id()
        .times(2)
        .returning(|_id| Ok(USER.clone()));

    contacts_dao_mock
        .expect_create()
        .times(1)
        .returning(|_uid, _user, _phone_number| Ok(()));

    blacklist_dao_mock.expect_get().times(2).returning(|_| {
        Ok(vec![BlacklistDao {
            id: Uuid::new_v4(),
            created_at: chrono::Utc::now().naive_local(),
            hash_blocker: hash("+4366412345678".to_string()),
            hash_blocked: hash("+4365012345678".to_string()),
        }])
    });

    contacts_dao_mock
        .expect_get_contacts()
        .times(1)
        .returning(|_user, _| {
            Ok(vec![ContactDto {
                blocked: false,
                name: "Test".to_string(),
                user: UserDto {
                    id: Uuid::new_v4(),
                    tele_num: "+4365012345678".to_string(),
                    led: true,
                    country_code: "AT".to_string(),
                    description: "doing something".to_string(),
                    changed_at: chrono::Utc::now().naive_local(),
                    profile_picture: "".to_string(),
                    hash_tele_num: hash("+4365012345678".to_string()),
                    xp: 0,
                    client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
                    access_token: None,
                    firebase_token: None,
                    session_token: None,
                },
            }])
        });

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contacts_dao_mock).await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/contacts/{}/{}", Uuid::new_v4(), "AT",))
        .set_json(&PayloadNumbersDto {
            numbers: vec![PayloadUserDto {
                name: "Test".to_string(),
                hash_tele_num: hash("+4366412345678"),
            }],
        })
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get()
        .uri(&format!("/api/contacts/{}", Uuid::new_v4(),))
        .to_request();

    let users: Vec<ContactDto> = test::read_response_json(&mut app, req).await;

    assert_eq!(1, users.len());
    assert_eq!(false, users.get(0).unwrap().user.led);
    assert_eq!("", &users.get(0).unwrap().user.description);

    //let resp = test::call_service(&mut app, req).await;
    //assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_contacts_with_blacklist_2() {
    let mut user_dao_mock = MockPersistentUserDao::new();
    let mut blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let mut contacts_dao_mock = MockPersistentContactsDao::new();

    user_dao_mock
        .expect_get_by_id()
        .times(2)
        .returning(|_id| Ok(USER.clone()));

    contacts_dao_mock
        .expect_create()
        .times(1)
        .returning(|_uid, _user, _phone_number| Ok(()));

    blacklist_dao_mock.expect_get().times(3).returning(|_| {
        Ok(vec![BlacklistDao {
            id: Uuid::new_v4(),
            created_at: chrono::Utc::now().naive_local(),
            hash_blocked: hash("+4366412345678".to_string()), //reversed
            hash_blocker: hash("+4365012345678".to_string()),
        }])
    });

    contacts_dao_mock
        .expect_get_contacts()
        .times(1)
        .returning(|_user, _| {
            Ok(vec![ContactDto {
                blocked: false,
                name: "Test".to_string(),
                user: UserDto {
                    id: Uuid::new_v4(),
                    tele_num: "+4365012345678".to_string(),
                    led: true,
                    country_code: "AT".to_string(),
                    description: "doing something".to_string(),
                    changed_at: chrono::Utc::now().naive_local(),
                    profile_picture: "".to_string(),
                    hash_tele_num: hash("+4365012345678".to_string()),
                    xp: 0,
                    client_version: super::ALLOWED_CLIENT_VERSIONS[0].to_string(),
                    access_token: None,
                    firebase_token: None,
                    session_token: None,
                },
            }])
        });

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contacts_dao_mock).await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/contacts/{}/{}", Uuid::new_v4(), "AT",))
        .set_json(&PayloadNumbersDto {
            numbers: vec![PayloadUserDto {
                name: "Test".to_string(),
                hash_tele_num: hash("+4366412345678"),
            }],
        })
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get()
        .uri(&format!("/api/contacts/{}", Uuid::new_v4(),))
        .to_request();

    let users: Vec<ContactDto> = test::read_response_json(&mut app, req).await;

    assert_eq!(1, users.len());
    assert_eq!(false, users.get(0).unwrap().user.led);
    assert_eq!("", &users.get(0).unwrap().user.description);

    //let resp = test::call_service(&mut app, req).await;
    //assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_contacts_max() {
    let user_dao_mock = MockPersistentUserDao::new();
    let blacklist_dao_mock = MockPersistentBlacklistDao::new();
    let contacts_dao_mock = MockPersistentContactsDao::new();

    let mut app = init_server!(user_dao_mock, blacklist_dao_mock, contacts_dao_mock).await;

    let len = 100000;
    let mut numbers = Vec::with_capacity(len);

    for _ in 0..len {
        numbers.push(PayloadUserDto {
            name: "Test".to_string(),
            hash_tele_num: hash("+4366412345678"),
        });
    }

    let req = test::TestRequest::post()
        .uri(&format!("/api/contacts/{}/{}", Uuid::new_v4(), "AT",))
        .set_json(&PayloadNumbersDto { numbers })
        .to_request();

    let resp = test::call_service(&mut app, req).await;
    assert!(resp.status().is_client_error());
}
