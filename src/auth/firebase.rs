use crate::auth::*;
use core::models::PhoneNumber;

#[derive(Debug, Clone)]
pub struct FirebaseDatabaseConfiguration {
    pub firebase_project_id: String,
    pub firebase_auth_token: String,
}

impl AuthenticatorConfiguration for FirebaseDatabaseConfiguration {
    fn get_project_id(&self) -> &String {
        &self.firebase_project_id
    }

    fn get_auth_token(&self) -> &String {
        &self.firebase_project_id
    }
}

/// Response of firebase for auth requests
#[derive(Debug, Deserialize)]
struct FirebaseAuthResponse {
    tele_num: String,
}

#[derive(Debug, Clone)]
pub struct FirebaseAuthenticator {
    pub config: FirebaseDatabaseConfiguration 
}

impl Authenticator for FirebaseAuthenticator {
    fn get_configuration(&self) -> Box<&dyn AuthenticatorConfiguration> {
        Box::new(&self.config)
    }

    fn authentification(
        &self,
        tele_num: &PhoneNumber,
        user_token: &String,
    ) -> Result<bool, ServiceError> {
        info!("auth/firebase");

        //FIXME
        let client = Client::new();
        let result: FirebaseAuthResponse = client
            .get(&format!(
                "https://{}.firebaseio.com/users/{}/.json?auth={}",
                self.config.get_project_id(), user_token, self.config.get_auth_token()
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
}
