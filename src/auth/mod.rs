use core::errors::ServiceError;
use core::models::PhoneNumber;
use log::{error, info};
use reqwest::Client;

pub mod firebase;

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

pub trait AuthenticatorConfiguration : Send + Sync {
    fn get_project_id(&self) -> &String;
    fn get_auth_token(&self) -> &String;
}

pub trait Authenticator : Send + Sync {
    fn authentification(
        &self,
        tele_num: &PhoneNumber,
        user_token: &String,
    ) -> Result<bool, ServiceError>;
    fn get_configuration(&self) -> Box<&dyn AuthenticatorConfiguration>;
}

#[macro_export]
macro_rules! authenticate_user {
    ($tele_num:expr, $uid:expr, $auth:expr) => {{
        let is_ok = $auth.authenticator.authentification($tele_num, $uid)?;

        if !is_ok {
            log::warn!(
                "Authentication failed for {} given firebase_uid {}",
                $tele_num.to_string(),
                $uid
            );
            Err(ServiceError::Unauthorized)
        } else {
            log::info!("Authentication ok");
            Ok(())
        }
    }};
}

#[macro_export]
macro_rules! authenticate_user_by_uid {
    ($id:expr, $key:expr, $auth:expr, $pool:expr) => {{
        let user = crate::queries::user::get_query($id, $pool)?;
        let tele = core::models::PhoneNumber::my_from(&user.tele_num, &user.country_code)?;

        let _ = authenticate_user!(&tele, $key, $auth)?;

        Ok(user)
    }};
}
