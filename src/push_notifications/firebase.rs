use crate::push_notifications::*;
use core::errors::ServiceError;

use tokio;
use core::models::{Analytic, Blacklist, PhoneNumber, UsageStatisticEntry, User, Contact};
use log::{error, info};
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

impl NotificationService for FirebaseNotificationService {
    fn push(&self, values: Vec<(User, Contact)>) -> Result<(), ServiceError> {
        let client = Client::new();

        let api_token = self.config.fcm_token.clone();
            let work = futures::stream::iter_ok(values)
            .map(move |(user, contact)| {
                //FIXME
                client
                    .post("https://fcm.googleapis.com/fcm/send")
                    .header(CONTENT_TYPE, "application/json")
                    .header(AUTHORIZATION, format!("key={}", api_token))
                    .json(&json!({
                        "notification": {
                            "title": format!("{} ist motiviert", contact.name),
                            "body": "",
                            "icon": "ic_stat_name_nougat"
                        },
                        "android":{
                            "ttl":"43200s"
                        },
                        "registration_ids": [user.firebase_token]
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
