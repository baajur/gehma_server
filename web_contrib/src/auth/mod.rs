use core::errors::ServiceError;
use core::models::PhoneNumber;
use log::{error, info};
use reqwest::Client;

pub mod testing;
pub mod twilio;

pub type Auth = AuthenticatorWrapper;

/// The actix's routes cannot handle `web::Data<impl Authenticator + 'static>`.
/// That's why I use a wrapper struct to make it easier to guess the type
pub struct AuthenticatorWrapper {
    pub authenticator: Box<dyn Authenticator>,
}

impl AuthenticatorWrapper {
    pub fn new(a: Box<dyn Authenticator>) -> Self {
        AuthenticatorWrapper { authenticator: a }
    }
}

pub trait Authenticator: Send + Sync {
    fn check_code(&self, tele_num: &PhoneNumber, user_token: &String)
        -> Result<bool, ServiceError>;

    fn request_code(&self, tele_num: &PhoneNumber) -> Result<(), ServiceError>;
    //fn get_configuration(&self) -> Box<&dyn AuthenticatorConfiguration>;
}

#[macro_export]
macro_rules! get_user_by_tele_num {
    ( $dao:ident, $tele_num:expr, $access_token:expr ) => {{
        $dao.get_ref()
            .get_by_tele_num($tele_num, $access_token)
            .map_err(|w| {
                log::error!("{:?}", w);
                ServiceError::Unauthorized
            })
    }};
}

#[macro_export]
macro_rules! get_user_by_id {
    ( $dao:ident, $expr:expr, $access_token:expr ) => {{
        $dao.get_ref()
            .get_by_id($tele_num, $access_token)
            .map_err(|w| {
                log::error!("{:?}", w);
                ServiceError::Unauthorized
            })
    }};
}
