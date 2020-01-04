use crate::push_notifications::*;
use core::errors::ServiceError;
use super::FirebaseToken;

use tokio;
use core::models::{User, Contact};
use log::{error};
use futures::Future;
use serde_json::json;

use futures::stream::Stream;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::r#async::Client;

#[derive(Debug, Clone)]
pub struct FirebaseConfiguration {
    pub fcm_token: String,
}

pub struct FirebaseNotificationService {
    pub config: FirebaseConfiguration
}

type Name = String;
impl NotificationService for FirebaseNotificationService {
    fn push(&self, values: Vec<(Name, FirebaseToken)>) -> Result<(), ServiceError> {
        let client = Client::new();

        let api_token = self.config.fcm_token.clone();
            let work = futures::stream::iter_ok(values)
            .map(move |(name, token)| {
                //FIXME implement return
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
                        "android":{
                            "ttl":"43200s"
                        },
                        "registration_ids": [token]
                    }))
                    .send()
            })
            .buffer_unordered(10)
            .and_then(|mut res| {
                //FIXME fix error
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

                println!("Response: {}", res.status());
                futures::future::ok(res.json::<serde_json::Value>())
            })
            .for_each(|_| Ok(()))
            .map_err(|e| error!("{}", e));

            tokio::run(work);

            Ok(())
    }
}
