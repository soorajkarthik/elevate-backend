use bcrypt::verify;
use rocket::http::Status;

use crate::models::auth::{BasicAuth, generate_token, TokenType};
use crate::models::database::PGConnection;
use crate::models::user::User;
use crate::views::request::StandardResponse;

#[post("/")]
pub fn login(auth: BasicAuth, mut connection: PGConnection) -> StandardResponse {
    let mut transaction = transaction!(connection);

    let user = match User::from_email(auth.username.clone(), &mut transaction) {
        Some(user) => user,
        None => return StandardResponse {
            status: Status::BadRequest,
            response: json!({
                "message": format!("No user found with email: {}", auth.username)
            }),
        }
    };

    if let Some(password) = auth.password {
        match verify(password, user.password.as_str()) {
            Ok(is_correct) => {
                if is_correct {
                    let token = match generate_token(user.email.clone(), TokenType::Auth) {
                        Ok(token) => token,
                        Err(err) => {
                            error!("{}", err);
                            StandardResponse {
                                status: Status::UnprocessableEntity,
                                response: json!({
                                    "message": "Couldn't generate auth token"
                                }),
                            }
                        }
                    };

                    StandardResponse {
                        status: Status::Ok,
                        response: json!({
                            "user": user,
                            "token": token
                        }),
                    }
                } else {
                    StandardResponse {
                        status: Status::BadRequest,
                        response: json!({
                            "message": "Incorrect password"
                        }),
                    }
                }
            }
            Err(err) => {
                error!("{}", err);
                StandardResponse {
                    status: Status::UnprocessableEntity,
                    response: json!({
                        "message": "Couldn't compare password with hash"
                    }),
                }
            }
        }
    } else {
        StandardResponse {
            status: Status::BadRequest,
            response: json!({
                "message": "No password was provided!"
            }),
        }
    }
}