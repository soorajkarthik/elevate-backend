use crate::models::alerts::{Alert, AlertType};
use crate::models::auth::{BearerToken, TokenType};
use crate::models::database::PGConnection;
use crate::models::user::User;
use crate::services::firebase::send_alert_notification;
use crate::services::mapquest::get_address;
use crate::views::request::StandardResponse;
use crate::{fetch_user, transaction};
use rocket::http::Status;
use rocket_contrib::json::Json;

#[get("/types")]
pub fn get_alert_types(token: BearerToken, mut connection: PGConnection) -> StandardResponse {
    let mut transaction = transaction!(connection);
    fetch_user!(token.token, TokenType::Auth, &mut transaction);

    StandardResponse {
        status: Status::Ok,
        response: json!(AlertType::get_all(&mut transaction)),
    }
}

#[post("/", format = "application/json", data = "<alert>")]
pub fn create_alert(
    alert: Json<Alert>,
    token: BearerToken,
    mut connection: PGConnection,
) -> StandardResponse {
    let mut transaction = transaction!(connection);
    let user = fetch_user!(token.token, TokenType::Auth, &mut transaction);

    let mut alert = alert.into_inner();
    alert.created_by = user.email;
    alert.place = get_address(alert.latitude, alert.longitude);

    let alert = match alert.init(&mut transaction) {
        Ok(alert) => alert,
        Err(_) => {
            return StandardResponse {
                status: Status::UnprocessableEntity,
                response: json!({
                    "message": "Could not create new alert"
                }),
            }
        }
    };

    let user_tokens = alert.nearby_user_tokens(&mut transaction);

    match transaction.commit() {
        Ok(_) => {
            let count = send_alert_notification(&alert, user_tokens);

            StandardResponse {
                status: Status::Ok,
                response: json!({
                    "message": format!("Alert successfully created, notified {} nearby users", count),
                    "alert": alert
                }),
            }
        }

        Err(err) => {
            error!("{}", err);
            StandardResponse {
                status: Status::ServiceUnavailable,
                response: json!({
                    "message": "Unable to commit changes to database"
                }),
            }
        }
    }
}

#[put("/<alert_id>", format = "application/json", data = "<updated>")]
pub fn update_alert(
    alert_id: i64,
    updated: Json<Alert>,
    token: BearerToken,
    mut connection: PGConnection,
) -> StandardResponse {
    let mut transaction = transaction!(connection);
    let user = fetch_user!(token.token, TokenType::Auth, &mut transaction);

    let alert = match Alert::get_by_id(alert_id, &mut transaction) {
        Some(alert) => alert,
        None => {
            return StandardResponse {
                status: Status::BadRequest,
                response: json!({
                    "message": format!("Could not find alert with id {}", alert_id)
                }),
            }
        }
    };

    if alert.created_by != user.email {
        return StandardResponse {
            status: Status::BadRequest,
            response: json!({
                "message": "You cannot edit an alert you did not create"
            }),
        };
    }

    let mut updated = updated.into_inner();
    updated.id = alert.id;

    if alert.latitude != updated.latitude || alert.longitude != updated.longitude {
        updated.place = get_address(alert.latitude, alert.longitude);
    }

    let updated = match updated.update(&mut transaction) {
        Ok(alert) => alert,
        Err(_) => {
            return StandardResponse {
                status: Status::UnprocessableEntity,
                response: json!({
                    "message": "Could not update alert"
                }),
            }
        }
    };

    match transaction.commit() {
        Ok(_) => StandardResponse {
            status: Status::Ok,
            response: json!({
                "message": "Alert successfully updated",
                "alert": updated
            }),
        },

        Err(err) => {
            error!("{}", err);
            StandardResponse {
                status: Status::ServiceUnavailable,
                response: json!({
                    "message": "Unable to commit changes to database"
                }),
            }
        }
    }
}

#[post("/<alert_id>/resolve")]
pub fn resolve_alert(
    alert_id: i64,
    token: BearerToken,
    mut connection: PGConnection,
) -> StandardResponse {
    let mut transaction = transaction!(connection);
    let user = fetch_user!(token.token, TokenType::Auth, &mut transaction);

    let alert = match Alert::get_by_id(alert_id, &mut transaction) {
        Some(alert) => alert,
        None => {
            return StandardResponse {
                status: Status::BadRequest,
                response: json!({
                    "message": format!("Could not find alert with id {}", alert_id)
                }),
            }
        }
    };

    if alert.created_by != user.email {
        return StandardResponse {
            status: Status::BadRequest,
            response: json!({
                "message": "You cannot edit an alert you did not create"
            }),
        };
    }

    let alert = match alert.resolve(&mut transaction) {
        Ok(alert) => alert,
        Err(_) => {
            return StandardResponse {
                status: Status::UnprocessableEntity,
                response: json!({
                    "message": "Couldn't mark alert as resolved"
                }),
            }
        }
    };

    match transaction.commit() {
        Ok(_) => StandardResponse {
            status: Status::Ok,
            response: json!({
                "message": "Alert successfully resolved",
                "alert": alert
            }),
        },

        Err(err) => {
            error!("{}", err);
            StandardResponse {
                status: Status::ServiceUnavailable,
                response: json!({
                    "message": "Unable to commit changes to database"
                }),
            }
        }
    }
}

#[delete("/<alert_id>")]
pub fn delete_alert(
    alert_id: i64,
    token: BearerToken,
    mut connection: PGConnection,
) -> StandardResponse {
    let mut transaction = transaction!(connection);
    let user = fetch_user!(token.token, TokenType::Auth, &mut transaction);

    let alert = match Alert::get_by_id(alert_id, &mut transaction) {
        Some(alert) => alert,
        None => {
            return StandardResponse {
                status: Status::BadRequest,
                response: json!({
                    "message": format!("Could not find alert with id {}", alert_id)
                }),
            }
        }
    };

    if alert.created_by != user.email {
        return StandardResponse {
            status: Status::BadRequest,
            response: json!({
                "message": "You cannot edit an alert you did not create"
            }),
        };
    }

    if alert.delete(&mut transaction).is_err() {
        return StandardResponse {
            status: Status::UnprocessableEntity,
            response: json!({
                "message": "Could not delete alert"
            }),
        };
    }

    match transaction.commit() {
        Ok(_) => StandardResponse {
            status: Status::Ok,
            response: json!({
                "message": "Successfully deleted alert",
                "alert": alert
            }),
        },

        Err(err) => {
            error!("{}", err);
            StandardResponse {
                status: Status::ServiceUnavailable,
                response: json!({
                    "message": "Unable to commit changes to database"
                }),
            }
        }
    }
}

#[get("/?<ne_lat>&<ne_lng>&<sw_lat>&<sw_lng>")]
pub fn get_by_viewport(
    ne_lat: f32,
    ne_lng: f32,
    sw_lat: f32,
    sw_lng: f32,
    token: BearerToken,
    mut connection: PGConnection,
) -> StandardResponse {
    let mut transaction = transaction!(connection);
    fetch_user!(token.token, TokenType::Auth, &mut transaction);

    StandardResponse {
        status: Status::Ok,
        response: json!(Alert::get_by_viewport(
            ne_lat,
            ne_lng,
            sw_lat,
            sw_lng,
            &mut transaction
        )),
    }
}
