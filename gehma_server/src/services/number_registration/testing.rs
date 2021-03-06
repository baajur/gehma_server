use super::NumberRegistrationServiceTrait;
use core::errors::ServiceError;
use core::models::PhoneNumber;
use log::{info};

#[derive(Debug, Clone)]
pub struct TestingAuthConfiguration {
    pub id: String,
    pub auth_token: String,
}


#[derive(Debug, Clone)]
pub struct TestingAuthentificator {
    pub config: TestingAuthConfiguration,
}

#[derive(Debug, Clone)]
pub struct TestingAuthentificatorAlwaysFalse {
    pub config: TestingAuthConfiguration,
}

impl NumberRegistrationServiceTrait for TestingAuthentificator {
    fn request_code(&self, _tele_num: &PhoneNumber) -> Result<(), ServiceError> {
        Ok(())
    }

    fn check_code(
        &self,
        _tele_num: &PhoneNumber,
        _user_token: &str,
    ) -> Result<bool, ServiceError> {
        info!("auth/testing");

        Ok(true)
    }
}

impl NumberRegistrationServiceTrait for TestingAuthentificatorAlwaysFalse {
    fn request_code(&self, _tele_num: &PhoneNumber) -> Result<(), ServiceError> {
        Ok(())
    }

    fn check_code(
        &self,
        _tele_num: &PhoneNumber,
        _user_token: &str,
    ) -> Result<bool, ServiceError> {
        info!("auth/testing");

        Ok(false)
    }
}
