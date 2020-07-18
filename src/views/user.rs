use bcrypt::verify;
use rocket::http::Status;
use rocket_contrib::json::Json;

use crate::models::auth::{BasicAuth, generate_token, TokenType};
use crate::models::database::PGConnection;
use crate::models::user::User;
use crate::views::request::StandardResponse;

#[post("/login")]
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

    if !user.verified {
        return StandardResponse {
            status: Status::PreconditionRequired,
            response: json!({
                "message": format!("Please verify your account using the link we sent to: {}", user.email.as_str())
            }),
        };
    }

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
                            "message": "Login successful!",
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

#[post("/", format = "application/json", data = "user_init")]
pub fn create_user(user_init: Json<User>, mut connection: PGConnection) -> StandardResponse {
    let mut transaction = transaction!(connection);

    let new_user = match user_init.init(&mut transaction) {
        Ok(user) => user,
        Err(_) => return StandardResponse {
            status: Status::BadRequest,
            response: json!({
                "message": format!("User already registered with email {}", new_user.email.as_str())
            }),
        }
    };

    match transaction.commit() {
        Ok(_) => {
            send_email_using_file!(
                new_user.email.as_str(),
                "Welcome to Elevate! Please verify your email.",
                "/emails/welcome.html",
                "{}",
                verification_token.as_str()
            );

            StandardResponse {
                status: Status::Created,
                response: json!({
                    "message": format!("User created successfully. Verification email sent to {}", new_user.email.as_str()),
                    "user": new_user,
                    "verificationToken": verification_token
                }),
            }
        }

        Err(_) => {
            StandardResponse {
                status: Status::ServiceUnavailable,
                response: json!({
                    "message": "Unable to commit changes to database"
                }),
            }
        }
    }
}