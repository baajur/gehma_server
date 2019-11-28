use crate::Pool;
use actix_web::web;
use core::errors::ServiceError;
use core::models::{Analytic, Blacklist, PhoneNumber, UsageStatisticEntry, User};
use diesel::{prelude::*, PgConnection};
use futures::Future;
use serde_json::json;
use tokio;
use uuid::Uuid;

use crate::routes::user::UpdateUser;

use log::{error, info};

pub(crate) fn get_entry_by_tel_query(
    tele: &PhoneNumber,
    pool: &web::Data<Pool>,
) -> Result<User, ::core::errors::ServiceError> {
    info!("queries/user/get_entry_by_tel_query");

    use core::schema::users::dsl::*;

    let conn: &PgConnection = &pool.get().unwrap();

    let tele = tele.to_string();

//    dbg!(&tele);

    let res = users
        .filter(tele_num.eq(tele))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|result| {
            Ok(result)
            //Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })?;

    res.first()
        .cloned()
        .ok_or_else(|| ServiceError::BadRequest("No user found".into()))
}

pub(crate) fn analytics_user(
    pool: &web::Data<Pool>,
    user: &User,
) -> Result<Analytic, ::core::errors::ServiceError> {
    info!("queries/user/analytics_user");
    use core::schema::analytics::dsl::analytics;

    let ana = Analytic::my_from(user);
    let conn: &PgConnection = &pool.get().unwrap();

    let w = diesel::insert_into(analytics)
        .values(&ana)
        .get_result(conn)
        .map_err(|_db_error| {
            eprintln!("{}", _db_error);
            ServiceError::BadRequest("Could not log change".into())
        })?;

    Ok(w)
}

pub(crate) fn create_query(
    tel: &PhoneNumber,
    country_code: &str,
    version: &str,
    access_token: &str,
    pool: &web::Data<Pool>,
) -> Result<User, ServiceError> {
    info!("queries/user/create_query");
    use core::schema::users::dsl::users;

    let new_inv: User = User::my_from(&tel.to_string(), country_code, version, access_token);
    let conn: &PgConnection = &pool.get().unwrap();

    let ins = diesel::insert_into(users)
        .values(&new_inv)
        .get_result(conn)?;

    //dbg!(&ins);

    Ok(ins)
}

pub(crate) fn analytics_usage_statistics(
    pool: &web::Data<Pool>,
    user: &User,
) -> Result<UsageStatisticEntry, ::core::errors::ServiceError> {
    info!("queries/user/analytics_usage_statistics");
    use core::schema::usage_statistics::dsl::usage_statistics;

    let ana = UsageStatisticEntry::my_from(user);
    let conn: &PgConnection = &pool.get().unwrap();

    let w = diesel::insert_into(usage_statistics)
        .values(&ana)
        .get_result(conn)
        .map_err(|_db_error| {
            eprintln!("{}", _db_error);
            ServiceError::BadRequest("Could not log change".into())
        })?;

    Ok(w)
}

pub(crate) fn update_user_query(
    myid: Uuid,
    user: &UpdateUser,
    pool: &web::Data<Pool>,
) -> Result<User, ::core::errors::ServiceError> {
    info!("queries/user/update_user_query");
    use core::schema::users::dsl::{
        changed_at, client_version, description, id, led, users,
    };

    let conn: &PgConnection = &pool.get().unwrap();

    let target = users.filter(id.eq(myid));

    let my_led = match &*user.led {
        "true" => true,
        "false" => false,
        _ => false,
    };

    diesel::update(target)
        .set((
            description.eq(user.description.to_string()),
            led.eq(my_led),
            changed_at.eq(chrono::Local::now().naive_local()),
            client_version.eq(user.client_version.clone()),
        ))
        .execute(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Updating state failed".into()))?;

    users
        .filter(id.eq(myid))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|res_users| {
            Ok(res_users
                .first()
                .cloned()
                .ok_or_else(|| ServiceError::BadRequest("No user found".into()))?)
        })
        .and_then(|user| {
            if my_led {
                //Sending push notification
                sending_push_notifications(&user, pool).map_err(|err| {
                    eprintln!("{}", err);
                    ServiceError::BadRequest("Cannot send push notifications".to_string())
                })?;
            }

            Ok(user)
        })
}

fn sending_push_notifications(user: &User, pool: &web::Data<Pool>) -> Result<(), ServiceError> {
    info!("queries/user/sending_push_notifications");
    use core::models::Contact;
    use core::schema::blacklist::dsl::{blacklist, blocked, blocker};
    use core::schema::contacts::dsl::{contacts, target_tele_num};
    use core::schema::users::dsl::{id, users};
    use futures::stream::Stream;
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
    use reqwest::r#async::Client;

    let conn: &PgConnection = &pool.get().unwrap();

    let mut contacts_who_saved_user = contacts
        .filter(target_tele_num.eq(&user.tele_num))
        .load::<Contact>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))?;

    dbg!(&contacts_who_saved_user);

    let my_filtered_contacts = blacklist
        .filter(blocker.eq(&user.tele_num).or(blocked.eq(&user.tele_num)))
        .load::<Blacklist>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))?;

    dbg!(&my_filtered_contacts);

    for i in my_filtered_contacts.iter() {
        let r = contacts_who_saved_user.iter().position(|c| {
            (c.target_tele_num == i.blocker && c.from_tele_num == i.blocked)
                || (c.target_tele_num == i.blocked && c.from_tele_num == i.blocker)
        });

        if let Some(r) = r {
            contacts_who_saved_user.remove(r);
        }
    }

    contacts_who_saved_user.sort_by(|a, b| a.from_id.partial_cmp(&b.from_id).unwrap());

    dbg!(&contacts_who_saved_user);

    let targets: Vec<_> = contacts_who_saved_user.iter().map(|w| &w.from_id).collect();

    let mut user_contacts: Vec<_> = users
        .filter(id.eq_any(targets))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|result| {
            Ok(result)
            //Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })?
        .into_iter()
        .filter(|w| w.firebase_token.is_some())
        //.map(|w| w.firebase_token.unwrap())
        //.take(crate::LIMIT_PUSH_NOTIFICATION_CONTACTS)
        .collect();

    user_contacts.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());

    //println!("{:#?}", user_contacts);
    //println!("{:#?}", contacts_who_saved_user);

    //FIXME extract to .data()
    let api_token = std::env::var("FCM_TOKEN").expect("No FCM_TOKEN configured");

    let client = Client::new();

    let test = user_contacts
        .clone()
        .into_iter()
        .zip(contacts_who_saved_user.clone())
        .take(crate::LIMIT_PUSH_NOTIFICATION_CONTACTS);

    if test.len() == 0 {
        info!("Nix zu senden");
    }

    for (user, contact) in test {
        //assert_eq!(user.id, contact.from_id);
        info!("{} ist motiviert zu {}", contact.name, user.tele_num);
    }

    let work = futures::stream::iter_ok(
        user_contacts
            .into_iter()
            .zip(contacts_who_saved_user)
            .take(crate::LIMIT_PUSH_NOTIFICATION_CONTACTS),
    )
    .map(move |(user, contact)| {
        //FIXME
        client
            .post("https://fcm.googleapis.com/fcm/send")
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("key={}", api_token))
            .json(&json!({
                "notification": {
                    "title": format!("{} ist motiviert", contact.name),
                    "body": "",
                    "icon": "ic_stat_name_nougat"
                },
                "android":{
                    "ttl":"43200s"
                },
                "registration_ids": [user.firebase_token]
            }))
            .send()
    })
    .buffer_unordered(10)
    .and_then(|mut res| {
        println!("Response: {}", res.status());
        futures::future::ok(res.json::<serde_json::Value>())
    })
    .for_each(|_| Ok(()))
    .map_err(|e| error!("{}", e));

    tokio::run(work);

    Ok(())
}

/// Get the user by uid
pub(crate) fn get_query(myid: Uuid, my_access_token: &str, pool: &web::Data<Pool>) -> Result<User, ServiceError> {
    info!("queries/user/get_query");
    use core::schema::users::dsl::{id, users, access_token};

    let conn: &PgConnection = &pool.get().unwrap();

    users
        .filter(id.eq(myid).and(access_token.eq(my_access_token)))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|w| {
            w.first()
                .cloned()
                .ok_or_else(|| ServiceError::BadRequest("No user found".into()))
        })
}

/// Get the user by uid
pub(crate) fn get_user_by_tele_num(phone_number: &PhoneNumber, my_access_token: &str, pool: &web::Data<Pool>) -> Result<User, ServiceError> {
    info!("queries/user/get_query");
    use core::schema::users::dsl::{tele_num, users, access_token};

    let conn: &PgConnection = &pool.get().unwrap();

    users
        .filter(tele_num.eq(phone_number.to_string()).and(access_token.eq(my_access_token)))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|w| {
            w.first()
                .cloned()
                .ok_or_else(|| ServiceError::BadRequest("No user found".into()))
        })
}

pub(crate) fn update_profile_picture(
    uid: Uuid,
    ending: String,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    info!("queries/user/update_profile_picture");
    use core::schema::users::dsl::{changed_at, id, profile_picture, users};

    let conn: &PgConnection = &pool.get().unwrap();

    let target = users.filter(id.eq(uid));

    diesel::update(target)
        .set((
            changed_at.eq(chrono::Local::now().naive_local()),
            profile_picture.eq(format!("static/profile_pictures/{}.{}", uid, ending)),
        ))
        .execute(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Updating state failed".into()))?;

    info!(
        "Updating profile {}",
        format!("static/profile_pictures/{}.{}", uid, ending)
    );

    Ok(())
}
