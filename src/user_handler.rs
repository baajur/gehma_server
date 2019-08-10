use crate::errors::ServiceError;
use crate::models::{Analytic, PhoneNumber, Pool, UsageStatisticEntry, User, Blacklist};
use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;
use serde_json::json;
use tokio;
use uuid::Uuid;

pub fn add(
    _info: web::Path<()>,
    body: web::Json<PostUser>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    dbg!(&body);
    web::block(move || create_entry(body.into_inner(), pool)).then(|res| match res {
        Ok(user) => {
            let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(user);
            crate::utils::set_response_headers(&mut res);
            Ok(res)
        }
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

pub fn get(
    info: web::Path<(String)>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    dbg!(&info);
    web::block(move || get_entry(&info.into_inner(), pool)).then(|res| match res {
        Ok(users) => {
            let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(users);
            crate::utils::set_response_headers(&mut res);
            Ok(res)
        }
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

#[derive(Debug, Deserialize)]
pub struct PostUser {
    pub tele_num: String,
    pub country_code: String,
    pub client_version: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub description: String,
    pub led: String,
    pub is_autofahrer: Option<String>,
    pub client_version: String,
}

pub fn update(
    info: web::Path<(String)>,
    data: web::Json<UpdateUser>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    dbg!(&info);
    dbg!(&data);
    web::block(move || update_user(&info.into_inner(), &data.into_inner(), &pool)).then(|res| {
        match res {
            Ok(user) => Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(&user)),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            },
        }
    })
}

fn create_entry(
    body: PostUser,
    pool: web::Data<Pool>,
) -> Result<User, crate::errors::ServiceError> {
    dbg!(&body);

    if !crate::ALLOWED_CLIENT_VERSIONS.contains(&body.client_version.as_str()) {
        return Err(ServiceError::BadRequest(format!(
            "Version mismatch. The supported versions are {:?}",
            crate::ALLOWED_CLIENT_VERSIONS
        )));
    }

    let tele = &body.tele_num;
    let country_code = &body.country_code;

    let tele2 = PhoneNumber::my_from(&tele, country_code)?;

    dbg!(&tele2.to_string());

    let user = match create_query(&tele2, &country_code, &body.client_version, &pool) {
        Ok(u) => Ok(u),
        Err(ServiceError::AlreadyExists(_)) => get_entry_by_tel_query(&tele2, &pool),
        Err(err) => Err(err),
    }?;

    if user.client_version != body.client_version {
        update_user(
            &user.id.to_string(),
            &UpdateUser {
                description: user.description.clone(),
                led: format!("{}", user.led),
                is_autofahrer: Some(format!("{}", user.is_autofahrer)),
                client_version: body.client_version.clone(),
            },
            &pool,
        )?;
    }

    dbg!(&user);

    analytics_usage_statistics(&pool, &user)?;

    Ok(user)
}

fn get_entry(uid: &String, pool: web::Data<Pool>) -> Result<User, crate::errors::ServiceError> {
    let parsed = Uuid::parse_str(uid)?;
    let users = get_query(parsed, &pool)?;
    dbg!(&users);

    let user = match users.len() {
        0 => Err(ServiceError::BadRequest("No user found".to_string())),
        _ => Ok(users.get(0).unwrap().clone()),
    }?;

    //analytics_usage_statistics(&pool, &user)?; not logging every refresh

    Ok(user)
}

fn get_entry_by_tel_query(
    tele: &PhoneNumber,
    pool: &web::Data<Pool>,
) -> Result<User, crate::errors::ServiceError> {
    use crate::schema::users::dsl::*;

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

fn update_user(
    uid: &String,
    user: &UpdateUser,
    pool: &web::Data<Pool>,
) -> Result<User, crate::errors::ServiceError> {
    let parsed = Uuid::parse_str(uid)?;
    let user = update_user_query(parsed, user, &pool)?;

    dbg!(&user);

    analytics_user(&pool, &user)?;

    Ok(user)
}

fn analytics_user(
    pool: &web::Data<Pool>,
    user: &User,
) -> Result<Analytic, crate::errors::ServiceError> {
    use crate::schema::analytics::dsl::analytics;

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

fn create_query(
    tel: &PhoneNumber,
    country_code: &String,
    version: &String,
    pool: &web::Data<Pool>,
) -> Result<User, crate::errors::ServiceError> {
    use crate::schema::users::dsl::users;

    let new_inv: User = User::my_from(&tel.to_string(), country_code, version);
    let conn: &PgConnection = &pool.get().unwrap();

    let ins = diesel::insert_into(users)
        .values(&new_inv)
        .get_result(conn)?;

    dbg!(&ins);

    Ok(ins)
}

fn analytics_usage_statistics(
    pool: &web::Data<Pool>,
    user: &User,
) -> Result<UsageStatisticEntry, crate::errors::ServiceError> {
    use crate::schema::usage_statistics::dsl::usage_statistics;

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

fn update_user_query(
    myid: Uuid,
    user: &UpdateUser,
    pool: &web::Data<Pool>,
) -> Result<User, crate::errors::ServiceError> {
    use crate::schema::users::dsl::{
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
                sending_push_notifications(&user, pool);
            }

            Ok(user)
        });

    db_user
}

fn sending_push_notifications(
    user: &User,
    pool: &web::Data<Pool>,
) -> Result<(), crate::errors::ServiceError> {
    use crate::models::Contact;
    use crate::schema::blacklist::dsl::{blacklist, blocker, blocked};
    use crate::schema::contacts::dsl::{contacts, from_id, target_tele_num};
    use crate::schema::users::dsl::{id, tele_num, users};
    use futures::stream::Stream;
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
    use reqwest::r#async::{Client, Response};

    let conn: &PgConnection = &pool.get().unwrap();

    let mut my_contacts = contacts
        .filter(target_tele_num.eq(&user.tele_num))
        .load::<Contact>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))?;

    let my_filtered_contacts = blacklist
        .filter(blocker.eq(&user.tele_num).and(blocked.eq(&user.tele_num)))
        .load::<Blacklist>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))?;

    for ignoring in my_filtered_contacts.iter() {
        if ignoring.blocker == user.tele_num {
            let index = my_contacts.iter().position(|x| *x.target_tele_num == ignoring.blocker);
            if let Some(index) = index {
                my_contacts.remove(index);
            }
        }
        else if ignoring.blocked == user.tele_num {
            let index = my_contacts.iter().position(|x| *x.target_tele_num == user.tele_num);
            if let Some(index) = index {
                my_contacts.remove(index);
            }
        }
    }

    my_contacts.sort_by(|a, b| a.from_id.partial_cmp(&b.from_id).unwrap());

    let targets: Vec<_> = my_contacts.iter().map(|w| &w.from_id).collect();

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
    println!("{:#?}", my_contacts);

    let api_token = std::env::var("FCM_TOKEN").expect("No FCM_TOKEN configured");

    let client = Client::new();

    let test = user_contacts
            .clone()
            .into_iter()
            .zip(my_contacts.clone())
            .take(crate::LIMIT_PUSH_NOTIFICATION_CONTACTS);

    for (user, contact) in test {
        assert_eq!(user.id, contact.from_id);
        println!("{} ist motiviert zu {}", contact.name, user.tele_num);
    }

    let work = futures::stream::iter_ok(
        user_contacts
            .into_iter()
            .zip(my_contacts)
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

fn get_query(myid: Uuid, pool: &web::Data<Pool>) -> Result<Vec<User>, crate::errors::ServiceError> {
    use crate::schema::users::dsl::{id, users};

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
