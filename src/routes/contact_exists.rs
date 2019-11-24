use actix_web::{error::BlockingError, web, HttpResponse};
use futures::Future;

use crate::Pool;
use core::errors::ServiceError;
use core::models::DowngradedUser;

use crate::auth::Auth;
use crate::utils::QueryParams;

use crate::controllers::contact_exists::get_entry;

use log::info;

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
    auth: web::Data<Auth>,
) -> impl Future<Item = HttpResponse, Error = ServiceError> {
    info!("controllers/contact_exists/exists");

    web::block(move || {
        let info = info.into_inner();
        get_entry(
            &info.0,
            &info.1,
            &mut payload.numbers,
            pool,
            &query.firebase_uid,
            auth,
        )
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
