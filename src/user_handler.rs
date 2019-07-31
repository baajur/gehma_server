use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;
use uuid::Uuid;

use crate::errors::ServiceError;
use crate::models::{Pool, User, PhoneNumber, Analytic, UsageStatisticEntry};

pub fn add(
    _info: web::Path<()>,
    body: web::Json<PostUser>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    dbg!(&body);
    web::block(move || create_entry(body.into_inner(), pool)).then(|res| match res {
        Ok(user) => Ok(HttpResponse::Ok().content_type("application/json").json(user)),
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
        Ok(users) => Ok(HttpResponse::Ok().content_type("application/json").json(&users)),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}


#[derive(Debug, Deserialize)]
pub struct PostUser{
    pub tele_num: String,
    pub country_code: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub description: String,
    pub led: String,
    pub is_autofahrer: Option<String>,
}

pub fn update(
    info: web::Path<(String)>,
    data: web::Json<UpdateUser>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    dbg!(&info);
    dbg!(&data);
    web::block(move || update_user(&info.into_inner(), &data.into_inner(), pool)).then(
        |res| match res {
            Ok(user) => Ok(HttpResponse::Ok().content_type("application/json").json(&user)),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            },
        },
    )
}

fn create_entry(
    body: PostUser,
    pool: web::Data<Pool>,
) -> Result<User, crate::errors::ServiceError> {
    dbg!(&body);
    let tele = body.tele_num;
    let country_code = &body.country_code;

    let tele2 = PhoneNumber::my_from(&tele, country_code)?;

    dbg!(&tele2.to_string());

    let user = match create_query(&tele2, &country_code, &pool) {
        Ok(u) => Ok(u),
        Err(ServiceError::AlreadyExists(_)) => get_entry_by_tel_query(&tele2, &pool),
        Err(err) => Err(err),
    }?;

    dbg!(&user);

    analytics_usage_statistics(&pool, &user)?;

    Ok(user)
}

fn get_entry(
    uid: &String,
    pool: web::Data<Pool>,
) -> Result<User, crate::errors::ServiceError> {
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
    pool: web::Data<Pool>,
) -> Result<User, crate::errors::ServiceError> {
    let parsed = Uuid::parse_str(uid)?;
    let users = update_user_query(parsed, user, &pool)?;

    dbg!(&users);

    let res = users.first()
        .map(|w| w.clone())
        .ok_or(ServiceError::BadRequest("No user found".into()))?;

    analytics_user(&pool, &res)?;

    Ok(res)
}

fn analytics_user(pool: &web::Data<Pool>, user: &User) -> Result<Analytic, crate::errors::ServiceError> {
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
    pool: &web::Data<Pool>,
) -> Result<User, crate::errors::ServiceError> {
    use crate::schema::users::dsl::users;

    let new_inv: User = User::my_from(&tel.to_string(), country_code);
    let conn: &PgConnection = &pool.get().unwrap();

    let ins = diesel::insert_into(users)
        .values(&new_inv)
        .get_result(conn)?;
                
    dbg!(&ins);

    Ok(ins)
}

fn analytics_usage_statistics(pool: &web::Data<Pool>, user: &User) -> Result<UsageStatisticEntry, crate::errors::ServiceError> {
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
) -> Result<Vec<User>, crate::errors::ServiceError> {
    use crate::schema::users::dsl::{description, is_autofahrer, led, id, users, changed_at};

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

    //FIXME update und danach query ist extrem teuer
    diesel::update(target)
        .set((
            description.eq(user.description.to_string()),
            led.eq(my_led),
            is_autofahrer.eq(my_is_autofahrer),
            changed_at.eq(chrono::Local::now().naive_local())
        ))
        .execute(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Updating state failed".into()))?;

    users
        .filter(id.eq(myid))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|result| {
            Ok(result)
            //Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })
}

fn get_query(
    myid: Uuid,
    pool: &web::Data<Pool>,
) -> Result<Vec<User>, crate::errors::ServiceError> {
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
