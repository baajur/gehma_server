use crate::auth::*;
use chrono::prelude::*;
use core::models::PhoneNumber;
use serde::{Serialize, Deserialize};

//FIXME drop Debug
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct TwilioAuthenticator {
    pub config: TwilioConfiguration,
}

impl Authenticator for TwilioAuthenticator {
    fn check_code(
        &self,
        tele_num: &PhoneNumber,
        user_token: &String,
    ) -> Result<bool, ServiceError> {
        info!("auth/authenticate");

        let params = [
            ("To", tele_num.to_string()),
            ("Code", user_token.to_string()),
        ];

        //FIXME
        let client = Client::new();
        let result: TwilioVerificationCheckResponse = client
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
                error!("{:?}", w);
                ServiceError::BadRequest("Cannot parse twilio's response".to_string())
            })?
            .json()
            .map_err(|_| ServiceError::Unauthorized)?;

        if result.to == tele_num.to_string() && &result.status == "approved" && result.valid == true
        {
            return Ok(true);
        }

        Ok(false)
    }

    fn request_code(&self, tele_num: &PhoneNumber) -> Result<(), ServiceError> {
        info!("auth/request_code");

        let params = [
            ("To", tele_num.to_string()),
            ("Channel", "sms".to_string()),
        ];

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
