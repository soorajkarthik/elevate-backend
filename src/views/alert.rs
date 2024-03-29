use crate::models::alerts::{Alert, AlertType};
use crate::models::auth::{BearerToken, TokenType};
use crate::models::database::PGConnection;
use crate::models::user::User;
use crate::services::firebase::send_alert_notification;
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

    let alert = match alert.init(&mut transaction) {
        Ok(alert) => alert,
        Err(err) => {
            return StandardResponse {
                status: Status::UnprocessableEntity,
                response: json!({ "message": format!("Could not create new alert: {}", err) }),
            }
        }
    };

    let notification_info = alert.get_notification_info(&mut transaction);

    match transaction.commit() {
        Ok(_) => {
            let count = send_alert_notification(&alert, notification_info);

            StandardResponse {
                status: Status::Ok,
                response: json!({
                    "message": format!("Alert successfully created, notified {} nearby user(s)", count),
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

    let updated = match alert.update(&mut updated, &mut transaction) {
        Ok(alert) => alert,
        Err(err) => {
            return StandardResponse {
                status: Status::UnprocessableEntity,
                response: json!({ "message": format!("Could not update alert: {}", err) }),
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

#[get("/?<lat>&<lng>&<lat_delta>&<lng_delta>")]
pub fn get_by_viewport(
    lat: f32,
    lng: f32,
    lat_delta: f32,
    lng_delta: f32,
    token: BearerToken,
    mut connection: PGConnection,
) -> StandardResponse {
    let mut transaction = transaction!(connection);
    fetch_user!(token.token, TokenType::Auth, &mut transaction);

    StandardResponse {
        status: Status::Ok,
        response: json!(Alert::get_by_viewport(
            lat + lat_delta / 2f32,
            lng + lng_delta / 2f32,
            lat - lat_delta / 2f32,
            lng - lng_delta / 2f32,
            &mut transaction
        )),
    }
}
