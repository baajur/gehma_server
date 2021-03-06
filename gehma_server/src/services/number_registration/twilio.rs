use super::NumberRegistrationServiceTrait;
use chrono::prelude::*;
use core::errors::ServiceError;
use core::models::PhoneNumber;
use log::{error, info};
use serde::{Deserialize, Serialize};

use reqwest::Client;

#[derive(Clone)]
pub struct TwilioConfiguration {
    pub project_id: String,
    pub account_id: String,
    pub auth_token: String,
}

impl TwilioConfiguration {
    pub fn get_project_id(&self) -> &String {
        &self.project_id
    }

    pub fn get_account_id(&self) -> &String {
        &self.account_id
    }

    pub fn get_auth_token(&self) -> &String {
        &self.auth_token
    }
}

/*
{
  "sid": "VEXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
  "service_sid": "VAXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
  "account_sid": "ACXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
  "to": "+15017122661",
  "channel": "sms",
  "status": "approved",
  "valid": true,
  "amount": null,
  "payee": null,
  "date_created": "2015-07-30T20:00:00Z",
  "date_updated": "2015-07-30T20:00:00Z"
}
*/

#[derive(Debug, Serialize, Deserialize)]
struct TwilioVerificationCheckResponse {
    sid: String,
    service_sid: String,
    account_sid: String,
    to: String,
    channel: String,
    status: String,
    valid: bool,
    amount: Option<String>,
    payee: Option<String>,
    date_created: chrono::DateTime<Utc>,
    date_updated: chrono::DateTime<Utc>,
}

/// Request a code
#[derive(Debug, Serialize, Deserialize)]
struct TwilioVerificationResponse {
    sid: String,
    service_sid: String,
    account_sid: String,
    to: String,
    channel: String,
    status: String,
    valid: bool,
    amount: Option<String>,
    payee: Option<String>,
    date_created: chrono::DateTime<Utc>,
    date_updated: chrono::DateTime<Utc>,
    url: String,
    //lookup removed on purpose
}

#[derive(Clone)]
pub struct TwilioAuthenticator {
    pub config: TwilioConfiguration,
}

impl NumberRegistrationServiceTrait for TwilioAuthenticator {
    fn check_code(&self, tele_num: &PhoneNumber, user_token: &str) -> Result<bool, ServiceError> {
        info!("fn check_code");

        let params = [
            ("To", tele_num.to_string()),
            ("Code", user_token.to_string()),
        ];

        info!("params {:?}", params);

        //FIXME
        let client = Client::new();
        let result: Result<TwilioVerificationCheckResponse, _> = client
            .post(&format!(
                "https://verify.twilio.com/v2/Services/{}/VerificationCheck",
                self.config.get_project_id()
            ))
            .form(&params)
            .basic_auth(
                self.config.get_account_id(),
                Some(self.config.get_auth_token()),
            )
            .send()
            .map_err(|w| {
                error!("error {:?}", w);
                eprintln!("error {:?}", w);
                ServiceError::BadRequest("Cannot parse twilio's response".to_string())
            })?
            .json()
            .map_err(|_| ServiceError::Unauthorized);

        info!("result {:?}", result);

        if let Ok(result) = result {
            //https://www.twilio.com/docs/verify/api
            if result.to == tele_num.to_string()
                && &result.status == "approved"
                && result.valid == true
            {
                info!("Check ok");
                return Ok(true);
            }
        }

        info!("Check not ok");

        Ok(false)
    }

    fn request_code(&self, tele_num: &PhoneNumber) -> Result<(), ServiceError> {
        info!("auth/request_code");

        let params = [("To", tele_num.to_string()), ("Channel", "sms".to_string())];

        //FIXME
        let client = Client::new();
        let _result: TwilioVerificationResponse = client
            .post(&format!(
                "https://verify.twilio.com/v2/Services/{}/Verifications",
                self.config.get_project_id()
            ))
            .form(&params)
            .basic_auth(
                self.config.get_account_id(),
                Some(self.config.get_auth_token()),
            )
            .send()
            .map_err(|w| {
                error!("{:?}", w);
                ServiceError::BadRequest("Cannot parse twilio's response".to_string())
            })?
            .json()
            .map_err(|_| ServiceError::Unauthorized)?;

        Ok(())
    }
}
