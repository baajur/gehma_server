use crate::Pool;
use ::core::errors::ServiceError;
use ::core::models::{Analytic, PhoneNumber, UsageStatisticEntry, User, Blacklist};
use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;
use serde_json::json;
use tokio;
use uuid::Uuid;

use crate::controllers::user::{PostUser, UpdateUser};

pub(crate) fn get_entry_by_tel_query(
    tele: &PhoneNumber,
    pool: &web::Data<Pool>,
) -> Result<User, ::core::errors::ServiceError> {
    use ::core::schema::users::dsl::*;

    let conn: &PgConnection = &pool.get().unwrap();

    let tele = tele.to_string();

    dbg!(&tele);

    let res = users
        .filter(tele_num.eq(tele))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|result| {
            Ok(result)
            //Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })?;

    res.first()
        .map(|w| w.clone())
        .ok_or(ServiceError::BadRequest("No user found".into()))
}

pub(crate) fn analytics_user(
    pool: &web::Data<Pool>,
    user: &User,
) -> Result<Analytic, ::core::errors::ServiceError> {
    use ::core::schema::analytics::dsl::analytics;

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
    country_code: &String,
    version: &String,
    pool: &web::Data<Pool>,
) -> Result<User, ServiceError> {
    use ::core::schema::users::dsl::users;

    let new_inv: User = User::my_from(&tel.to_string(), country_code, version);
    let conn: &PgConnection = &pool.get().unwrap();

    let ins = diesel::insert_into(users)
        .values(&new_inv)
        .get_result(conn)?;

    dbg!(&ins);

    Ok(ins)
}

pub(crate) fn analytics_usage_statistics(
    pool: &web::Data<Pool>,
    user: &User,
) -> Result<UsageStatisticEntry, ::core::errors::ServiceError> {
    use ::core::schema::usage_statistics::dsl::usage_statistics;

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
    use ::core::schema::users::dsl::{
        changed_at, client_version, description, id, is_autofahrer, led, users,
    };

    let conn: &PgConnection = &pool.get().unwrap();

    let target = users.filter(id.eq(myid));

    let my_led = match &*user.led {
        "true" => true,
        "false" => false,
        _ => false,
    };

    let my_is_autofahrer = match user.is_autofahrer.as_ref().map(|w| &**w) {
        Some("true") => true,
        Some("false") => false,
        _ => false,
    };

    diesel::update(target)
        .set((
            description.eq(user.description.to_string()),
            led.eq(my_led),
            is_autofahrer.eq(my_is_autofahrer),
            changed_at.eq(chrono::Local::now().naive_local()),
            client_version.eq(user.client_version.clone()),
        ))
        .execute(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Updating state failed".into()))?;

    let db_user = users
        .filter(id.eq(myid))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|res_users| {
            Ok(res_users
                .first()
                .map(|w| w.clone())
                .ok_or(ServiceError::BadRequest("No user found".into()))?)
        })
        .and_then(|user| {
            if my_led {
                //Sending push notification
                sending_push_notifications(&user, pool)
                    .map_err(|err| { eprintln!("{}", err); ServiceError::BadRequest("Cannot send push notifications".to_string()) })?;
            }

            Ok(user)
        });

    db_user
}

fn sending_push_notifications(
    user: &User,
    pool: &web::Data<Pool>,
) -> Result<(), ServiceError> {
    use ::core::models::Contact;
    use ::core::schema::blacklist::dsl::{blacklist, blocker, blocked};
    use ::core::schema::contacts::dsl::{contacts, from_id, target_tele_num};
    use ::core::schema::users::dsl::{id, tele_num, users};
    use futures::stream::Stream;
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
    use reqwest::r#async::{Client, Response};

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
        let r = contacts_who_saved_user
            .iter()
            .position(|c| (c.target_tele_num == i.blocker && c.from_tele_num == i.blocked) 
                        || (c.target_tele_num == i.blocked && c.from_tele_num == i.blocker));

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

    println!("{:#?}", user_contacts);
    println!("{:#?}", contacts_who_saved_user);

    let api_token = std::env::var("FCM_TOKEN").expect("No FCM_TOKEN configured");

    let client = Client::new();

    let test = user_contacts
            .clone()
            .into_iter()
            .zip(contacts_who_saved_user.clone())
            .take(crate::LIMIT_PUSH_NOTIFICATION_CONTACTS);

    if test.len() == 0 {
        println!("Nix zu senden");
    }

    for (user, contact) in test {
        //assert_eq!(user.id, contact.from_id);
        println!("{} ist motiviert zu {}", contact.name, user.tele_num);
    }

    let work = futures::stream::iter_ok(
        user_contacts
            .into_iter()
            .zip(contacts_who_saved_user)
            .take(crate::LIMIT_PUSH_NOTIFICATION_CONTACTS),
    )
    .map(move |(user, contact)| {
        client
            .post("https://fcm.googleapis.com/fcm/send")
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("key={}", api_token))
            .json(&json!({
                "notification": {
                    "title": format!("{} ist motiviert", contact.name),
                    "body": ""
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
    .map_err(|e| eprintln!("{}", e));

    tokio::run(work);

    Ok(())
}

pub(crate) fn get_query(myid: Uuid, pool: &web::Data<Pool>) -> Result<Vec<User>, ServiceError> {
    use ::core::schema::users::dsl::{id, users};

    let conn: &PgConnection = &pool.get().unwrap();

    users
        .filter(id.eq(myid))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|result| {
            Ok(result)
            //Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })
}
