use crate::models::alerts::{Alert, AlertNotificationInfo};
use reqwest::blocking::Client;
use reqwest::Error;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};
use std::{env, thread, time};

#[derive(Debug, Serialize, Deserialize)]
struct FirebaseResponseMessageId {
    message_id: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FirebaseMultiCastResponse {
    multicast_id: i64,
    success: i64,
    failure: i64,
    canonical_ids: i64,
    results: Vec<FirebaseResponseMessageId>,
}

pub fn send_alert_notification(
    alert: &Alert,
    notification_info: Vec<AlertNotificationInfo>,
) -> u16 {
    let api_key = env::var("FIREBASE_MESSAGING_SERVER_KEY");

    if let Ok(api_key) = api_key {
        let client = Client::new();
        let url = String::from("https://fcm.googleapis.com/fcm/send");

        let payloads = notification_info
            .iter()
            .map(|info| {
                json!({
                    "data": {
                        "title": format!("Elevate {} Alert", &alert.alert_type),
                        "message": format!("{} reported near {:#?}! About {:.1} miles away", &alert.alert_type, &alert.place, &info.distance)
                    },
                    "to": info.token,
                    "priority": 10
                })
            })
            .collect::<Vec<JsonValue>>();

        debug!("Notification payloads: {:#?}", payloads);

        let mut count: u16 = 0;

        for payload in payloads {
            let response = client
                .post(&url)
                .json(&payload)
                .bearer_auth(&api_key)
                .send();
            match response {
                Ok(response) => {
                    let response_json: Result<FirebaseMultiCastResponse, Error> = response.json();
                    match response_json {
                        Ok(fmc_response) => {
                            info!("Multicast {} was successful", fmc_response.multicast_id);
                            count += 1;
                        }
                        Err(_) => {
                            error!("Failed to send notification.");
                        }
                    }
                }
                Err(msg) => {
                    error!("{}", msg);
                }
            };
            thread::sleep(time::Duration::from_millis(100));
        }

        return count;
    }

    return 0u16;
}
