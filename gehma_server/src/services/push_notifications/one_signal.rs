use super::Token;
use crate::services::push_notifications::*;
use core::errors::{InternalServerError, ServiceError};

use log::{debug, error, info};
use serde::Deserialize;
use serde_json::json;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;

#[derive(Clone)]
pub struct OneSignalConfiguration {
    pub id: String,
    pub key: String,
}

pub struct OneSignalService {
    pub config: OneSignalConfiguration,
}

/**
 *
 * {
    "id": "xxx",
    "recipients": 1,
    "external_id": null
}*/
#[derive(Debug, Deserialize)]
struct OneSignalResponse {
    id: String,
    recipients: u32,
}

type Name = String;
impl NotificationServiceTrait for OneSignalService {
    fn push(&self, values: Vec<(Name, Token)>) -> Result<(), ServiceError> {
        let client = Client::new();
        //let size : usize = values.len();

        let id = self.config.id.clone();
        let api_token = self.config.key.clone();

        if values.len() > 2000 {
            //Onesignal rate limit
            return Err(ServiceError::BadRequest(
                "Too many contacts. Ratelimit reached".to_string(),
            ));
        }

        let tokens: Vec<_> = values
            .into_iter()
            .filter(|w| w.1 != "".to_string())
            .map(|w| w.1)
            .collect();

        let response = client
            .post("https://onesignal.com/api/v1/notifications")
            .header(CONTENT_TYPE, "application/json")
            //.header(AUTHORIZATION, api_token.clone())
            .json(&json!({
                "app_id": id,
                "contents": {
                    "en": format!("Your friends are motivated"),
                    "de": format!("Deine Freunde sind motiviert"),
                },
                "ttl": 172800, //two days
                "include_player_ids": tokens
            }))
            .send()
            .map_err(|err| {
                error!("error {:#?}", err);
                ServiceError::InternalServerError(InternalServerError::NotificationError)
            });

        //info!("response {:#?}", response);

        /*
        let work = tokio::prelude::stream::iter_ok(values)
            .map(move |(name, token)| {
                info!("Send to {}", token);
                client
                    .post("https://fcm.googleapis.com/fcm/send")
                    .header(CONTENT_TYPE, "application/json")
                    .header(AUTHORIZATION, format!("key={}", api_token))
                    .json(&json!({
                        "notification": {
                            "title": format!("{} ist motiviert", name),
                            "body": "",
                            "icon": "ic_stat_name_nougat"
                        },
                        "priority": "high",
                        "registration_ids": [token]
                    }))
                    .send()
                    .map_err(|err| {
                error!("error {:?}", err);
                ServiceError::InternalServerError(InternalServerError::NotificationError)
            })})
            .buffer_unordered(10)
            .and_then(|mut res| {
                res.json::<FirebaseResponse>().map_err(|w| {
                    error!("FirebaseResponse {:?}", w);
                    ServiceError::InternalServerError(InternalServerError::NotificationError)
                })
            })
            .for_each(move |res| {
                if res.success != 1 {
                    error!("SOME NOTIFICATIONS FAILED");
                    error!("{:#?}", res);
                }

                Ok(())
            })
            .map_err(|e| {
                error!("error {:?}", e);
                //ServiceError::InternalServerError(InternalServerError::NotificationError)
                ()
            });

        tokio::run(work);
        */

        Ok(())
    }
}
