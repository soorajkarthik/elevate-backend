use std::fs::read_to_string;

use bcrypt::verify;
use rocket::http::Status;
use rocket_contrib::json::Json;

use crate::{send_email_using_file, transaction};
use crate::models::auth::{BasicAuth, BearerToken, generate_token, store_token, TokenType, validate_token};
use crate::models::database::PGConnection;
use crate::models::user::User;
use crate::views::request::StandardResponse;
use crate::views::send_email;

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
                            return StandardResponse {
                                status: Status::UnprocessableEntity,
                                response: json!({
                                    "message": "Couldn't generate auth token"
                                }),
                            };
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

#[post("/", format = "application/json", data = "<user_init>")]
pub fn create_user(user_init: Json<User>, mut connection: PGConnection) -> StandardResponse {
    let mut transaction = transaction!(connection);

    let new_user = match user_init.init(&mut transaction) {
        Ok(user) => user,
        Err(_) => return StandardResponse {
            status: Status::BadRequest,
            response: json!({
                "message": format!("User already registered with email {}", user_init.email.as_str())
            }),
        }
    };

    let verification_token = match generate_token(new_user.email.clone(), TokenType::Verification) {
        Ok(token) => token,
        Err(_) => return StandardResponse {
            status: Status::UnprocessableEntity,
            response: json!({
                "message": "Could not generate email verification token"
            }),
        }
    };

    let verification_token = match store_token(
        verification_token.clone(),
        TokenType::Verification,
        new_user.email.clone(),
        &mut transaction,
    ) {
        Ok(token) => token,
        Err(_) => return StandardResponse {
            status: Status::UnprocessableEntity,
            response: json!({
                "message": "Could not store email verification token"
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

#[post("/verify")]
pub fn verify_email(token: BearerToken, mut connection: PGConnection) -> StandardResponse {
    let mut transaction = transaction!(connection);

    let email = match validate_token(token.token, TokenType::Verification) {
        Ok(data) => data,
        Err(_) => return StandardResponse {
            status: Status::UnprocessableEntity,
            response: json!({
                "message": "Could not process verification token"
            }),
        }
    };

    let user = match User::from_email(email, &mut transaction) {
        Some(user) => user,
        None => return StandardResponse {
            status: Status::BadRequest,
            response: json!({
                "message": "Invalid verification token"
            }),
        }
    };

    let user = match user.verify_email(&mut transaction) {
        Ok(user) => user,
        Err(_) => return StandardResponse {
            status: Status::UnprocessableEntity,
            response: json!({
                "message": "Could not verify user email"
            }),
        }
    };

    match transaction.commit() {
        Ok(_) => StandardResponse {
            status: Status::Created,
            response: json!({
                "message": "User email successfully verified",
                "user": user,
            }),
        },

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