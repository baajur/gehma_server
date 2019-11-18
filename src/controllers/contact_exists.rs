use actix_web::{error::BlockingError, web, HttpResponse};
use futures::Future;
use uuid::Uuid;

use crate::Pool;
use core::errors::ServiceError;
use core::models::{DowngradedUser};

use core::models::User;
use crate::utils::QueryParams;
use crate::auth::FirebaseDatabaseConfiguration;

use log::{debug, info};

pub const MAX_ALLOWED_CONTACTS: usize = 10000;
pub const MIN_TELE_NUM_LENGTH: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseUser {
    pub calculated_tele: String,
    pub old: String,
    pub name: String,
    pub user: Option<DowngradedUser>,
}

#[derive(Debug, Deserialize)]
pub struct Payload {
    pub numbers: Vec<PayloadUser>,
}

#[derive(Debug, Deserialize)]
pub struct PayloadUser {
    pub name: String,
    pub tele_num: String,
}

pub fn exists(
    info: web::Path<(String, String)>,
    mut payload: web::Json<Payload>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/contact_exists/exists");

    web::block(move || {
        let info = info.into_inner();
        get_entry(&info.0, &info.1, &mut payload.numbers, pool, &query.firebase_uid, firebase_config)
    })
    .then(|res| match res {
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

fn get_entry(
    uid: &str,
    country_code: &str,
    phone_numbers: &mut Vec<PayloadUser>,
    pool: web::Data<Pool>,
    firebase_uid: &String,
    firebase_config: web::Data<FirebaseDatabaseConfiguration>,
) -> Result<Vec<ResponseUser>, ServiceError> {
    let parsed = Uuid::parse_str(uid)?;

    let user : Result<User, ServiceError> = authenticate_user_by_uid!(parsed, firebase_uid, firebase_config.into_inner(), &pool);

    user?;

    let users =
        crate::queries::contact_exists::get_query(parsed, phone_numbers, country_code, pool)?;

    Ok(users)
}
