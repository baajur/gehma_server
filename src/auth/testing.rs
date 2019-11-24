use crate::auth::*;
use core::models::PhoneNumber;
use core::errors::ServiceError;

#[derive(Debug, Clone)]
pub struct TestingAuthConfiguration {
    pub id: String,
    pub auth_token: String,
}

impl AuthenticatorConfiguration for TestingAuthConfiguration {
    fn get_project_id(&self) -> &String {
        &self.id
    }

    fn get_auth_token(&self) -> &String {
        &self.auth_token
    }
}

#[derive(Debug, Clone)]
pub struct TestingAuthentificator {
    pub config: TestingAuthConfiguration,
}

impl Authenticator for TestingAuthentificator {
    fn get_configuration(&self) -> Box<&dyn AuthenticatorConfiguration> {
        Box::new(&self.config)
    }

    fn authentification(
        &self,
        _tele_num: &PhoneNumber,
        _user_token: &String,
    ) -> Result<bool, ServiceError> {
        info!("auth/testing");

        Ok(true)
    }
}
