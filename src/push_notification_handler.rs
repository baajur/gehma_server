use actix_web::{error::BlockingError, web, HttpResponse};
use diesel::{prelude::*, PgConnection};
use futures::Future;
use uuid::Uuid;

use super::Pool;
use ::core::errors::ServiceError;
use ::core::models::{Analytic, PhoneNumber, UsageStatisticEntry, User};

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub token: String,
}

pub fn update_token(
    _info: web::Path<(String)>,
    body: web::Json<Payload>,
    pool: web::Data<Pool>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    dbg!(&body);
    web::block(move || update_token_handler(_info.into_inner(), body.into_inner(), pool)).then(
        |res| match res {
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
        },
    )
}

fn update_token_handler(
    uid: String,
    payload: Payload,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    let parsed = Uuid::parse_str(&uid)?;

    update_token_query(parsed, payload.token, &pool)
}

fn update_token_query(
    uid: Uuid,
    token: String,
    pool: &web::Data<Pool>,
) -> Result<(), ServiceError> {
    use ::core::schema::users::dsl::*;
    let conn: &PgConnection = &pool.get().unwrap();

    let target = users.filter(id.eq(uid));

    diesel::update(target)
        .set((
            firebase_token.eq(Some(token)),
            changed_at.eq(chrono::Local::now().naive_local()),
        ))
        .execute(conn)
        .map_err(|_db_error| {
            eprintln!("{}", _db_error);
            ServiceError::BadRequest("Updating state failed".into())
        })?;

    Ok(())
}
