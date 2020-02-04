use actix_web::{error::BlockingError, web, HttpResponse};
use futures::Future;

use crate::Pool;
use core::errors::ServiceError;
use core::models::{DowngradedUser, HashedTeleNum};

use web_contrib::auth::Auth;
use web_contrib::utils::{QueryParams, set_response_headers};

use crate::controllers::contact_exists::get_entry;

use log::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseUser {
    pub hash_tele_num: HashedTeleNum,
    pub name: String,
    pub user: Option<DowngradedUser>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Payload {
    pub numbers: Vec<PayloadUser>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PayloadUser {
    pub name: String,
    pub hash_tele_num: HashedTeleNum,
}

pub async fn exists(
    info: web::Path<(String, String)>,
    mut payload: web::Json<Payload>,
    pool: web::Data<Pool>,
    query: web::Query<QueryParams>,
    auth: web::Data<Auth>,
) -> HttpResponse {
    info!("controllers/contact_exists/exists");

    let info = info.into_inner();
        let users = get_entry(
            &info.0,
            &info.1,
            &mut payload.numbers,
            pool,
            &query.access_token,
            auth,
        ).unwrap();

    let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(users);
            //set_response_headers(&mut res);
            //Ok(res)
    res


    /*
    web::block(move || {
        let info = info.into_inner();
        get_entry(
            &info.0,
            &info.1,
            &mut payload.numbers,
            pool,
            &query.access_token,
            auth,
        )
    })
    .then(|res| match res {
        Ok(users) => {
            let mut res = HttpResponse::Ok()
                .content_type("application/json")
                .json(users);
            set_response_headers(&mut res);
            Ok(res)
        }
        Err(err) => match err {
            BlockingError::Error(service_error) => Err(service_error),
            BlockingError::Canceled => Err(ServiceError::InternalServerError),
        },
    })
    */
}
