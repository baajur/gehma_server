use core::errors::ServiceError;
use core::models::PhoneNumber;
use log::{error, info};
use reqwest::Client;

//pub mod firebase;
pub mod testing;
pub mod twilio;

pub type Auth = AuthenticatorWrapper;

/// The actix's routes cannot handle `web::Data<impl Authenticator + 'static>`.
/// That's why I use a wrapper struct to make it easier to guess the type
pub struct AuthenticatorWrapper {
    //pub authenticator: crate::auth::firebase::FirebaseAuthenticator
    pub authenticator: Box<dyn Authenticator>
}

impl AuthenticatorWrapper {
    pub fn new(a: Box<dyn Authenticator>) -> Self {
        AuthenticatorWrapper {
            authenticator: a
        }
    }
}

/*
pub trait AuthenticatorConfiguration : Send + Sync {
    fn get_project_id(&self) -> &String;
    fn get_auth_token(&self) -> &String;
}
*/

pub trait Authenticator : Send + Sync {
    fn check_code(
        &self,
        tele_num: &PhoneNumber,
        user_token: &String,
    ) -> Result<bool, ServiceError>;

    fn request_code(&self, tele_num: &PhoneNumber) -> Result<(), ServiceError>;
    //fn get_configuration(&self) -> Box<&dyn AuthenticatorConfiguration>;
}

#[macro_export]
macro_rules! get_user_by_tele_num {
    ($tele_num:expr, $access_token:expr, $auth:expr, $pool:expr) => {{
        crate::queries::user::get_user_by_tele_num($tele_num, $access_token, $pool)
            .map_err(|w| {
                log::error!("{:?}", w);
                ServiceError::Unauthorized
            })
    }};
}

#[macro_export]
macro_rules! get_user_by_id {
    ($id:expr, $access_token:expr, $auth:expr, $pool:expr) => {{
        let user = crate::queries::user::get_query($id, $access_token, $pool)?;
        //let tele = core::models::PhoneNumber::my_from(&user.tele_num, &user.country_code)?;
        //let _ = authenticate_user!(&tele, $key, $auth)?;

        Ok(user)
    }};
}
