use crate::models::alerts::AlertType;
use crate::models::auth::{BearerToken, TokenType};
use crate::models::database::PGConnection;
use crate::models::user::User;
use crate::views::request::StandardResponse;
use crate::{fetch_user, transaction};
use rocket::http::Status;

#[get("/types")]
pub fn get_alert_types(token: BearerToken, mut connection: PGConnection) -> StandardResponse {
    let mut transaction = transaction!(connection);
    fetch_user!(token.token, TokenType::Auth, &mut transaction);

    StandardResponse {
        status: Status::Ok,
        response: json!(AlertType::get_all(&mut transaction)),
    }
}
