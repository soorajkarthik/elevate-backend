use crate::models::alerts::Alert;
use reqwest::blocking::Client;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use std::env;

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

pub fn send_alert_notification(alert: &Alert, user_tokens: Vec<String>) -> i64 {
    let api_key = env::var("FIREBASE_MESSAGING_SERVER_KEY");

    if let Ok(api_key) = api_key {
        let client = Client::new();
        let url = String::from("https://fcm.googleapis.com/fcm/send");

        let payload = json!({
            "registration_ids": user_tokens,
            "notification": {
                "body": format!("{} reported near {}!", &alert.alert_type, &alert.place),
                "title": alert.alert_type
            }
        });

        let response = client.post(&url).json(&payload).bearer_auth(api_key).send();
        match response {
            Ok(response) => {
                let response_json: Result<FirebaseMultiCastResponse, Error> = response.json();
                match response_json {
                    Ok(fmc_response) => {
                        info!("Message sent successfully");
                        return fmc_response.success;
                    }
                    Err(_) => {
                        error!("Failed to send notification.");
                        return 0;
                    }
                }
            }
            Err(msg) => {
                error!("{}", msg);
                return 0;
            }
        };
    }

    return 0;
}
