use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;

use crate::errors::ServiceError;
use crate::models::{Pool, User};
use crate::utils::phonenumber_to_international;

pub fn add(
    info: web::Path<(String, String)>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    web::block(move || create_entry(&info.0, &info.1, pool)).then(|res| match res {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

pub fn get(
    info: web::Path<(String, String)>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    web::block(move || get_entry(&info.0, &info.1, pool)).then(|res| match res {
        Ok(users) => Ok(HttpResponse::Ok().json(&users)),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub description: String,
}

pub fn update(
    info: web::Path<(String, String)>,
    data: web::Json<UpdateUser>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    web::block(move || update_user(&info.0, &info.1, &data.into_inner(), pool)).then(
        |res| match res {
            Ok(_) => Ok(HttpResponse::Ok().finish()),
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            },
        },
    )
}

pub fn update_led(
    info: web::Path<(String, String, bool)>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    web::block(move || update_led_entry(&info.0, &info.1, &info.2, pool)).then(|res| match res {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
}

fn create_entry(
    tel: &String,
    country_code: &String,
    pool: web::Data<Pool>,
) -> Result<(), crate::errors::ServiceError> {
    dbg!(&tel);
    let _ = dbg!(create_query(tel, country_code, pool)?);
    Ok(())
}

fn get_entry(
    tele_num: &String,
    country_code: &String,
    pool: web::Data<Pool>,
) -> Result<User, crate::errors::ServiceError> {
    let users = get_query(tele_num, country_code, pool)?;
    dbg!(&users);

    match users.len() {
        0 => Err(ServiceError::BadRequest("No user found".to_string())),
        _ => Ok(users.get(0).unwrap().clone()),
    }
}

fn update_user(
    tel: &String,
    country_code: &String,
    user: &UpdateUser,
    pool: web::Data<Pool>,
) -> Result<(), crate::errors::ServiceError> {
    let _ = dbg!(update_user_query(tel, country_code, user, pool)?);
    Ok(())
}

fn update_led_entry(
    tel: &String,
    country_code: &String,
    led: &bool,
    pool: web::Data<Pool>,
) -> Result<(), crate::errors::ServiceError> {
    let _ = dbg!(update_led_query(tel, country_code, led, pool)?);
    Ok(())
}

fn create_query(
    tel: &String,
    country_code: &String,
    pool: web::Data<Pool>,
) -> Result<User, crate::errors::ServiceError> {
    use crate::schema::users::dsl::users;

    let new_inv: User = User::my_from(tel, country_code);
    let conn: &PgConnection = &pool.get().unwrap();

    let ins = diesel::insert_into(users)
        .values(&new_inv)
        .get_result(conn)?;

    Ok(ins)
}

fn update_user_query(
    tel: &String,
    country_code: &String,
    user: &UpdateUser,
    pool: web::Data<Pool>,
) -> Result<usize, crate::errors::ServiceError> {
    use crate::schema::users::dsl::{tele_num, users, description};

    let conn: &PgConnection = &pool.get().unwrap();

    let ttarget = phonenumber_to_international(&tel, &country_code)
        .replace("+", "")
        .replace(" ", "");

    let target = users.filter(tele_num.eq(ttarget));

    let res = diesel::update(target)
        .set(description.eq(user.description.to_string()))
        .execute(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Updating state failed".into()))?;

    if res != 200 {
        Err(ServiceError::BadRequest("Nothing updated".into()))
    } else {
        Ok(res)
    }
}

fn update_led_query(
    tele: &String,
    country_code: &String,
    bled: &bool,
    pool: web::Data<Pool>,
) -> Result<usize, crate::errors::ServiceError> {
    use crate::schema::users::dsl::{led, tele_num, users};

    let conn: &PgConnection = &pool.get().unwrap();

    let ttarget = phonenumber_to_international(&tele, &country_code)
        .replace("+", "")
        .replace(" ", "");

    let target = users.filter(tele_num.eq(ttarget));

    let res = diesel::update(target)
        .set(led.eq(bled))
        .execute(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Updating state failed".into()))?;

    if res != 200 {
        Err(ServiceError::BadRequest("Nothing updated".into()))
    } else {
        Ok(res)
    }
}

fn get_query(
    para_num: &String,
    country_code: &String,
    pool: web::Data<Pool>,
) -> Result<Vec<User>, crate::errors::ServiceError> {
    use crate::schema::users::dsl::{tele_num, users};

    let conn: &PgConnection = &pool.get().unwrap();

    let tel = phonenumber_to_international(&format!("+{}", para_num), &country_code)
        .chars()
        .into_iter()
        .skip(1)
        .collect::<String>();

    dbg!(&tel);

    users
        .filter(tele_num.eq(tel))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid User".into()))
        .and_then(|result| {
            Ok(result)
            //Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })
}
