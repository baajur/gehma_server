use actix_web::{error::BlockingError, web, HttpResponse};
use futures::Future;
use uuid::Uuid;

use crate::Pool;
use core::errors::ServiceError;
use core::models::{DowngradedUser};

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
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/contact_exists/exists");
    debug!("path {:?}", info);
    debug!("body {:?}", payload);

    web::block(move || {
        let info = info.into_inner();
        get_entry(&info.0, &info.1, &mut payload.numbers, pool)
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
    uid: &String,
    country_code: &String,
    phone_numbers: &mut Vec<PayloadUser>,
    pool: web::Data<Pool>,
) -> Result<Vec<ResponseUser>, ServiceError> {
    let parsed = Uuid::parse_str(uid)?;
    let users =
        crate::queries::contact_exists::get_query(parsed, phone_numbers, country_code, pool)?;

    Ok(users)
}
