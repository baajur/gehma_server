use core::errors::ServiceError;
use core::models::PhoneNumber;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use std::sync::Arc;
use log::{info, error};

#[derive(Debug, Clone)]
pub struct FirebaseDatabaseConfiguration {
    pub firebase_project_id: String,
    pub firebase_auth_token: String,
}

/// Response of firebase for auth requests
#[derive(Debug, Deserialize)]
struct FirebaseAuthResponse {
    tele_num: String,
}

#[macro_export]
macro_rules! authenticate_user {
    ($tele_num:expr, $firebase_uid:expr, $firebase_config:expr) => {{
        let is_ok =
            crate::auth::priv_authenticate_user($tele_num, $firebase_uid, $firebase_config)?;

        if !is_ok {
            log::warn!("Authentication failed for {} given firebase_uid {}", $tele_num.to_string(), $firebase_uid);
            Err(ServiceError::Unauthorized)
        }
        else {
            log::info!("Authentication ok");
            Ok(())
        }
    }};
}

#[macro_export]
macro_rules! authenticate_user_by_uid {
    ($uid:expr, $firebase_uid:expr, $firebase_config:expr, $pool:expr) => {{
        let user = crate::queries::user::get_query($uid, $pool)?;
        let tele = core::models::PhoneNumber::my_from(&user.tele_num, &user.country_code)?;

        let ok = authenticate_user!(&tele, $firebase_uid, $firebase_config)?;

        Ok(user)
    }}
}

pub fn priv_authenticate_user(
    tele_num: &PhoneNumber,
    user_given_firebase_uid: &String,
    firebase_config: Arc<FirebaseDatabaseConfiguration>,
) -> Result<bool, ServiceError> {
    info!("priv_authenticate_user");

    let config = firebase_config.clone();

    //FIXME
    let client = Client::new();
    let result: FirebaseAuthResponse = client
        .get(&format!(
            "https://{}.firebaseio.com/users/{}/.json?auth={}",
            config.firebase_project_id, user_given_firebase_uid, config.firebase_auth_token
        ))
        .send()
        .map_err(|w| {
            error!("{:?}", w);
            ServiceError::BadRequest("Cannot parse firebase's response".to_string())
        })?
        .json()
        .map_err(|_| ServiceError::Unauthorized)?;

    if result.tele_num == tele_num.to_string() {
        return Ok(true);
    }

    Ok(false)
}
