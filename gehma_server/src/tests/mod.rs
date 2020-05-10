pub use super::*;

use crate::ratelimits::{DefaultRateLimitPolicy, RateLimitWrapper};
use crate::services::number_registration::{
    NumberRegistrationService, NumberRegistrationServiceTrait,
};
use core::models::dto::*;
use data_encoding::HEXUPPER;
use ring::digest;

//mod index;
mod api;

pub(crate) fn set_testing_auth() -> NumberRegistrationService {
    let config = TestingAuthConfiguration {
        id: "test".to_string(),
        auth_token: "test".to_string(),
    };

    Box::new(TestingAuthentificator { config: config })
}

pub(crate) fn set_testing_notification_service() -> NotificationService {
    Box::new(TestingNotificationService)
}

pub(crate) fn set_ratelimits() -> RateLimitWrapper {
    RateLimitWrapper::new(Box::new(DefaultRateLimitPolicy))
}

fn hash(value: impl Into<String>) -> HashedTeleNum {
    HashedTeleNum(
        HEXUPPER.encode(digest::digest(&digest::SHA256, value.into().as_bytes()).as_ref()),
    )
}

#[cfg(feature = "integration_tests")]
mod integration;
