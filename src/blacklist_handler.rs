use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;
use uuid::Uuid;

use crate::errors::ServiceError;
use crate::models::{Blacklist, PhoneNumber, Pool, User};

#[derive(Debug, Deserialize)]
pub struct GetAllData {
    numbers: Vec<String>,
}

pub fn get_all(
    info: web::Path<(String)>,
    //data: web::Json<GetAllData>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    dbg!(&info);
    let info = info.into_inner();
    web::block(move || get_entry(&info, pool)).then(|res| match res {
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
pub struct PostData {
    blocked: String,
    country_code: String,
}

pub fn add(
    info: web::Path<(String)>,
    data: web::Json<PostData>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    dbg!(&info);
    dbg!(&data);
    web::block(move || create_entry(&info.into_inner(), &data.into_inner(), pool)).then(|res| {
        match res {
            Ok(_) => {
                let mut res = HttpResponse::Ok().content_type("application/json").finish();
                crate::utils::set_response_headers(&mut res);
                Ok(res)
            }
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            },
        }
    })
}

pub fn delete(
    info: web::Path<(String)>,
    data: web::Json<PostData>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    dbg!(&info);
    web::block(move || delete_entry(&info.into_inner(), &data.into_inner(), pool)).then(|res| {
        match res {
            Ok(_) => {
                let mut res = HttpResponse::Ok().content_type("application/json").finish();
                crate::utils::set_response_headers(&mut res);
                Ok(res)
            }
            Err(err) => match err {
                BlockingError::Error(service_error) => Err(service_error),
                BlockingError::Canceled => Err(ServiceError::InternalServerError),
            },
        }
    })
}

fn get_entry(
    blocker: &String,
    pool: web::Data<Pool>,
) -> Result<Vec<Blacklist>, crate::errors::ServiceError> {
    let blocker = Uuid::parse_str(blocker)?;

    let bl = get_query(blocker, pool)?;

    dbg!(&bl);

    Ok(bl)
}

fn get_query(
    sblocker: Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<Blacklist>, crate::errors::ServiceError> {
    use crate::schema::blacklist::dsl::{blacklist, blocker};
    use crate::schema::users::dsl::{id, users};

    let conn: &PgConnection = &pool.get().unwrap();

    let user = users
        .filter(id.eq(sblocker))
        .load::<User>(conn)
        .map_err(|_db_err| ServiceError::BadRequest("Invalid User".into()))?
        .first()
        .map(|w| w.clone())
        .ok_or(ServiceError::BadRequest("No user found".into()))?;

    blacklist
        .filter(blocker.eq(user.tele_num))
        .load::<Blacklist>(conn)
        .map_err(|_db_err| ServiceError::BadRequest("Invalid User".into()))
}

fn create_entry(
    blocker: &String,
    data: &PostData,
    pool: web::Data<Pool>,
) -> Result<Blacklist, crate::errors::ServiceError> {
    use crate::schema::users::dsl::{id, users};

    let blocker2 = Uuid::parse_str(blocker)?;
    let blocked = PhoneNumber::my_from(&data.blocked, &data.country_code)?;

    dbg!(&blocked);
    dbg!(&blocker);

    let conn: &PgConnection = &pool.get().unwrap();

    let myusers = users
        .filter(id.eq(blocker2))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot find user".into()))?;

    let user = myusers
        .first()
        .map(|w| w.clone())
        .ok_or(ServiceError::BadRequest(
            "No user found with given uid".into(),
        ))?;

    let tel = PhoneNumber::my_from(&user.tele_num, &data.country_code)?;
    let b = create_query(&tel, &blocked, pool)?;

    dbg!(&b);
    Ok(b)
}

fn create_query(
    blocker: &PhoneNumber,
    blocked: &PhoneNumber,
    pool: web::Data<Pool>,
) -> Result<Blacklist, crate::errors::ServiceError> {
    use crate::schema::blacklist::dsl::blacklist;

    let conn: &PgConnection = &pool.get().unwrap();
    let new_inv: Blacklist = Blacklist::my_from(blocker, blocked);

    let ins = diesel::insert_into(blacklist)
        .values(&new_inv)
        .get_result(conn)?;
    //.map_err(|_db_error| ServiceError::BadRequest("Cannot insert into blacklist".into()))?;

    dbg!(&ins);

    Ok(ins)
}

fn delete_entry(
    blocker: &String,
    data: &PostData,
    pool: web::Data<Pool>,
) -> Result<(), crate::errors::ServiceError> {
    use crate::schema::users::dsl::{id, users};

    let blocker2 = Uuid::parse_str(blocker)?;
    let blocked = PhoneNumber::my_from(&data.blocked, &data.country_code)?;

    let conn: &PgConnection = &pool.get().unwrap();

    let myusers = users
        .filter(id.eq(blocker2))
        .load::<User>(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot find user".into()))?;

    let user = myusers
        .first()
        .map(|w| w.clone())
        .ok_or(ServiceError::BadRequest(
            "No user found with given uid".into(),
        ))?;

    let tel = PhoneNumber::my_from(&user.tele_num, &data.country_code)?;

    delete_query(&tel, &blocked, pool)
}

fn delete_query(
    sblocker: &PhoneNumber,
    sblocked: &PhoneNumber,
    pool: web::Data<Pool>,
) -> Result<(), crate::errors::ServiceError> {
    use crate::schema::blacklist::dsl::{blacklist, blocked, blocker};
    let conn: &PgConnection = &pool.get().unwrap();

    let target = blacklist
        .filter(blocker.eq(sblocker.to_string()))
        .filter(blocked.eq(sblocked.to_string()));

    diesel::delete(target)
        .execute(conn)
        .map_err(|_db_error| ServiceError::BadRequest("Cannot delete blacklist".into()))?;

    Ok(())
}
