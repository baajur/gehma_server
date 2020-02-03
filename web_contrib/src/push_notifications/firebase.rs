use super::FirebaseToken;
use crate::push_notifications::*;
use core::errors::ServiceError;

use futures::Future;
use log::{error, info};
use serde::Deserialize;
use serde_json::json;
use tokio;

use futures::stream::Stream;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::r#async::Client;

#[derive(Debug, Clone)]
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
impl NotificationService for FirebaseNotificationService {
    fn push(&self, values: Vec<(Name, FirebaseToken)>) -> Result<(), ServiceError> {
        let client = Client::new();
        //let size : usize = values.len();

        let api_token = self.config.fcm_token.clone();
        let work = futures::stream::iter_ok(values)
            .map(move |(name, token)| {
                println!("Send to {}", name);
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
                        "priority": "high",
                        "registration_ids": [token]
                    }))
                    .send()
            })
            .buffer_unordered(10)
            .map_err(|w| {
                error!("{:?}", w);
                ServiceError::InternalServerError
            })
            .and_then(|mut res| {
                res.json::<FirebaseResponse>().map_err(|w| {
                    error!("FirebaseResponse {:?}", w);
                    ServiceError::InternalServerError
                })
            })
            .for_each(move |res| {
                if res.success != 1 {
                    error!("SOME NOTIFICATIONS FAILED");
                }

                Ok(())
            })
            .map_err(|e| error!("{}", e));

        tokio::run(work);

        Ok(())
    }
}
