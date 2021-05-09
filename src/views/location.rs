use crate::models::auth::{BearerToken, TokenType};
use crate::models::database::PGConnection;
use crate::models::location::Location;
use crate::models::user::User;
use crate::services::mapquest::{get_address, MapquestResult};
use crate::views::request::StandardResponse;
use crate::{fetch_user, transaction};
use rocket::http::Status;
use rocket_contrib::json::Json;

#[post("/location", format = "application/json", data = "<locations>")]
pub fn update_user_location(
    locations: Json<Vec<Location>>,
    token: BearerToken,
    mut connection: PGConnection,
) -> StandardResponse {
    let mut locations = locations.into_inner();

    if locations.len() <= 0 {
        return StandardResponse {
            status: Status::BadRequest,
            response: json!({
              "message": "No new locations given"
            }),
        };
    }

    let mut transaction = transaction!(connection);
    let user = fetch_user!(token.token, TokenType::Auth, &mut transaction);

    // Get latest location. Unwrap safe since we checked len > 0
    let mut location = locations.last_mut().unwrap();

    location.user_id = user.id;

    let location = match location.init_or_update(&mut transaction) {
        Ok(location) => location,
        Err(_) => {
            return StandardResponse {
                status: Status::UnprocessableEntity,
                response: json!({
                    "message": "Could not update user location information"
                }),
            }
        }
    };

    match transaction.commit() {
        Ok(_) => StandardResponse {
            status: Status::Ok,
            response: json!({
                "message": "Location updated successfully",
                "location": location
            }),
        },

        Err(_) => StandardResponse {
            status: Status::ServiceUnavailable,
            response: json!({
                "message": "Unable to commit changes to database"
            }),
        },
    }
}

#[get("/location")]
pub fn get_user_location(token: BearerToken, mut connection: PGConnection) -> StandardResponse {
    let mut transaction = transaction!(connection);
    let user = fetch_user!(token.token, TokenType::Auth, &mut transaction);

    StandardResponse {
        status: Status::Ok,
        response: json!(user.get_location(&mut transaction)),
    }
}

#[get("/address")]
pub fn get_user_address(token: BearerToken, mut connection: PGConnection) -> StandardResponse {
    let mut transaction = transaction!(connection);
    let user = fetch_user!(token.token, TokenType::Auth, &mut transaction);

    if let Some(location) = user.get_location(&mut transaction) {
        return match get_address(location.latitude, location.longitude) {
            MapquestResult::Success(address) => StandardResponse {
                status: Status::Ok,
                response: json!({ "address": address }),
            },
            MapquestResult::NoAPIKey | MapquestResult::NoValue | MapquestResult::NoResult => {
                StandardResponse {
                    status: Status::BadRequest,
                    response: json!({
                        "message": "Request to location reverse api failed"
                    }),
                }
            }
        };
    }

    StandardResponse {
        status: Status::BadRequest,
        response: json!({
            "message": "User location not found"
        }),
    }
}
