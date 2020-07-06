use crate::services::number_registration::testing::*;
use crate::services::number_registration::twilio::*;
use crate::services::number_registration::NumberRegistrationService;
use crate::services::push_notifications::firebase::*;
use crate::services::push_notifications::one_signal::*;
use crate::services::push_notifications::testing::*;
use crate::services::push_notifications::NotificationService;
use crate::services::session::*;

use crate::ratelimits::*;

#[allow(dead_code)]
pub(crate) fn get_auth() -> NumberRegistrationService {
    let project_id = std::env::var("TWILIO_PROJECT_ID").expect("no PROJECT_ID");
    let auth_token = std::env::var("TWILIO_AUTH_TOKEN").expect("no AUTH_TOKEN");
    let sid = std::env::var("TWILIO_ACCOUNT_ID").expect("no ACCOUNT_ID");

    let config = TwilioConfiguration {
        project_id,
        account_id: sid,
        auth_token,
    };

    Box::new(TwilioAuthenticator { config })
}

#[allow(dead_code)]
pub(crate) fn set_testing_auth() -> NumberRegistrationService {
    let config = TestingAuthConfiguration {
        id: "test".to_string(),
        auth_token: "test".to_string(),
    };

    Box::new(TestingAuthentificator { config })
}

#[allow(dead_code)]
pub(crate) fn set_testing_auth_false() -> NumberRegistrationService {
    let config = TestingAuthConfiguration {
        id: "test".to_string(),
        auth_token: "test".to_string(),
    };

    Box::new(TestingAuthentificatorAlwaysFalse { config })
}

#[allow(dead_code)]
pub(crate) fn set_testing_notification() -> NotificationService {
    Box::new(TestingNotificationService)
}

#[allow(dead_code)]
pub(crate) fn get_ratelimits() -> RateLimitWrapper {
    RateLimitWrapper::new(Box::new(DefaultRateLimitPolicy))
}

#[allow(dead_code)]
pub(crate) fn get_firebase_notification_service() -> NotificationService {
    let api_token = std::env::var("FCM_TOKEN").expect("No FCM_TOKEN configured");

    let config = FirebaseConfiguration {
        fcm_token: api_token,
    };

    Box::new(FirebaseNotificationService { config })
}

#[allow(dead_code)]
pub(crate) fn get_onesignal_notification_service() -> NotificationService {
    let id = std::env::var("ONE_SIGNAL_ID").expect("No ONE_SIGNAL_ID configured");
    let key = std::env::var("ONE_SIGNAL_KEY").expect("No ONE_SIGNAL_KEY configured");

    let config = OneSignalConfiguration { id, key };

    Box::new(OneSignalService { config })
}

#[allow(dead_code)]
pub(crate) fn get_session_service() -> SessionService {
    let secret = std::env::var("SESSION_KEY").expect("No SESSION_KEY configured");

    Box::new(SessionServicePriv::new(secret))
}

macro_rules! response {
    ($payload:expr) => {{
        use actix_web::{HttpResponse};

        let mut res = HttpResponse::Ok()
            .content_type("application/json")
            .json($expr);

        set_response_headers(&mut res);

        Ok(res)
    }};
}
