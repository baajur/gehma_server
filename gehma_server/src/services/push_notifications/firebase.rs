use super::FirebaseToken;
use crate::services::push_notifications::*;
use core::errors::{InternalServerError, ServiceError};

use log::{error, info};
use serde::Deserialize;
use serde_json::json;

use futures::stream::Stream;
use futures::Future;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;

#[derive(Clone)]
pub struct FirebaseConfiguration {
    pub fcm_token: String,
}

pub struct FirebaseNotificationService {
    pub config: FirebaseConfiguration,
}

/*{
    "multicast_id": 1715198469273987789,
    "success": 0,
    "failure": 1,
    "canonical_ids": 0,
    "results": [
        {
            "error": "NotRegistered"
        }
    ]
}*/

#[derive(Debug, Deserialize)]
struct FirebaseResponse {
    success: usize,
    failure: usize,
    canonical_ids: usize,
}

type Name = String;
impl NotificationServiceTrait for FirebaseNotificationService {
    fn push(&self, values: Vec<(Name, FirebaseToken)>) -> Result<(), ServiceError> {
        let client = Client::new();
        //let size : usize = values.len();

        let api_token = self.config.fcm_token.clone();

        let work = futures::stream::iter_ok(values)
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

        Ok(())
    }
}
