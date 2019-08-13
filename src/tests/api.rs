use super::*;
use reqwest;
use ::core::models::*;
use std::thread;

fn new_user() -> User {
let new_user = User::my_from(&"123456789".into(), &"AT".into(), &crate::ALLOWED_CLIENT_VERSIONS[0].into());

   let my_user : User = reqwest::Client::new()
        .post("http://localhost:3000/api/user")
        .json(&new_user)
        .send().unwrap()
        .json().unwrap(); 

    my_user
}

#[test]
fn test_post_user() {
    thread::spawn(|| {
        main();
    });

    thread::sleep_ms(500);

    let my_user = new_user();

    assert_eq!(my_user.tele_num, "+43123456789".to_string());
    assert_eq!(my_user.country_code, "AT".to_string());
}

#[test]
fn test_get_user() {
    thread::spawn(|| {
        main();
    });

    thread::sleep_ms(500);

    let database_user = new_user();

    let my_user : User = reqwest::Client::new()
        .get(&format!("http://localhost:3000/api/user/{}", database_user.id))
        .send().unwrap()
        .json().unwrap(); 

    assert_eq!(my_user, database_user);
}

#[test]
fn test_update_state() {
    thread::spawn(|| {
        main();
    });

    thread::sleep_ms(500);

    let database_user = new_user();

    reqwest::Client::new()
        .put(&format!("http://localhost:3000/api/user/{}", database_user.id))
        .json(&crate::controllers::user::UpdateUser {
            description: "test".into(),
            led: "true".into(),
            is_autofahrer: Some("false".into()),
            client_version: crate::ALLOWED_CLIENT_VERSIONS[0].to_string()
        })
        .send().unwrap();

    let my_user : User = reqwest::Client::new()
        .get(&format!("http://localhost:3000/api/user/{}", database_user.id))
        .send().unwrap()
        .json().unwrap(); 

    assert_eq!(my_user.description, "test".to_string());
    assert_eq!(my_user.led, true);
    assert_eq!(my_user.is_autofahrer, false);
    assert_eq!(my_user.client_version, crate::ALLOWED_CLIENT_VERSIONS[0].to_string());
}

#[test]
fn test_update_token() {
    thread::spawn(|| {
        main();
    });

    thread::sleep_ms(500);

    let database_user = new_user();

    reqwest::Client::new()
        .put(&format!("http://localhost:3000/api/user/{}/token", database_user.id))
        .json(&crate::controllers::push_notification::Payload {
            token: "updated token".to_string()
        })
        .send().unwrap();

    let my_user : User = reqwest::Client::new()
        .get(&format!("http://localhost:3000/api/user/{}", database_user.id))
        .send().unwrap()
        .json().unwrap(); 

    assert_eq!(my_user.firebase_token, Some("updated token".to_string()));
}