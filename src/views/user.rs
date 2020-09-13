use crate::models::auth::*;
use crate::models::database::PGConnection;
use crate::models::user::User;
use crate::views::request::StandardResponse;
use crate::views::send_email;
use crate::{fetch_user, send_email_using_file, transaction};
use bcrypt::verify;
use rocket::http::Status;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;

#[post("/login")]
pub fn login(auth: BasicAuth, mut connection: PGConnection) -> StandardResponse {
    let mut transaction = transaction!(connection);

    let user = match User::from_email(auth.username.clone(), &mut transaction) {
        Some(user) => user,
        None => {
            return StandardResponse {
                status: Status::BadRequest,
                response: json!({
                    "message": format!("No user found with email: {}", auth.username)
                }),
            }
        }
    };

    if !user.verified {
        return StandardResponse {
            status: Status::PreconditionRequired,
            response: json!({
                "message":
                    format!(
                        "Please verify your account using the link we sent to: {}",
                        user.email.as_str()
                    )
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
        Err(_) => {
            return StandardResponse {
                status: Status::BadRequest,
                response: json!({
                    "message":
                        format!(
                            "User already registered with email {}",
                            user_init.email.as_str()
                        )
                }),
            }
        }
    };

    let verification_token = match generate_token(new_user.email.clone(), TokenType::Verification) {
        Ok(token) => token,
        Err(_) => {
            return StandardResponse {
                status: Status::UnprocessableEntity,
                response: json!({
                    "message": "Could not generate email verification token"
                }),
            }
        }
    };

    let verification_token = match store_token(
        verification_token.clone(),
        TokenType::Verification,
        new_user.email.clone(),
        &mut transaction,
    ) {
        Ok(token) => token,
        Err(_) => {
            return StandardResponse {
                status: Status::UnprocessableEntity,
                response: json!({
                    "message": "Could not store email verification token"
                }),
            }
        }
    };

    match transaction.commit() {
        Ok(_) => {
            send_email_using_file!(
                new_user.email.as_str(),
                "Welcome to Elevate! Please verify your email.",
                "src/emails/welcome.html",
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

        Err(_) => StandardResponse {
            status: Status::ServiceUnavailable,
            response: json!({
                "message": "Unable to commit changes to database"
            }),
        },
    }
}

#[post("/verify")]
pub fn verify_email(token: BearerToken, mut connection: PGConnection) -> StandardResponse {
    let mut transaction = transaction!(connection);

    let token = match retrieve_token(token.token, &mut transaction) {
        Some(token) => token,
        None => {
            return StandardResponse {
                status: Status::BadRequest,
                response: json!({
                    "message": "Token has expired or as already been used"
                }),
            }
        }
    };

    let user = fetch_user!(token, TokenType::Verification, &mut transaction);

    let user = match user.verify_email(&mut transaction) {
        Ok(user) => user,
        Err(_) => {
            return StandardResponse {
                status: Status::UnprocessableEntity,
                response: json!({
                    "message": "Could not verify user email"
                }),
            }
        }
    };

    match transaction.commit() {
        Ok(_) => {
            send_email_using_file!(
                user.email.as_str(),
                "Thanks for verifying your email!",
                "src/emails/verification_confirmation.html",
                "Thank you for verifying your email"
            );

            StandardResponse {
                status: Status::Created,
                response: json!({
                    "message": "User email successfully verified",
                    "user": user,
                }),
            }
        }

        Err(_) => StandardResponse {
            status: Status::ServiceUnavailable,
            response: json!({
                "message": "Unable to commit changes to database"
            }),
        },
    }
}

#[get("/verify?<email>")]
pub fn send_verification_email(email: String, mut connection: PGConnection) -> StandardResponse {
    let mut transaction = transaction!(connection);

    let user = match User::from_email(email.clone(), &mut transaction) {
        Some(user) => user,
        None => {
            return StandardResponse {
                status: Status::BadRequest,
                response: json!({ "message": format!("No user found with email: {}", &email) }),
            }
        }
    };

    let verification_token = match generate_token(user.email.clone(), TokenType::Verification) {
        Ok(token) => token,
        Err(_) => {
            return StandardResponse {
                status: Status::UnprocessableEntity,
                response: json!({
                    "message": "Could not generate email verification token"
                }),
            }
        }
    };

    let verification_token = match store_token(
        verification_token.clone(),
        TokenType::Verification,
        user.email.clone(),
        &mut transaction,
    ) {
        Ok(token) => token,
        Err(_) => {
            return StandardResponse {
                status: Status::UnprocessableEntity,
                response: json!({
                    "message": "Could not store email verification token"
                }),
            }
        }
    };

    match transaction.commit() {
        Ok(_) => {
            send_email_using_file!(
                user.email.as_str(),
                "Please verify your email.",
                "src/emails/verification.html",
                "{}",
                verification_token.as_str()
            );

            StandardResponse {
                status: Status::Created,
                response: json!({
                    "message": format!("Verification email sent to {}", user.email.as_str()),
                    "verificationToken": verification_token
                }),
            }
        }

        Err(_) => StandardResponse {
            status: Status::ServiceUnavailable,
            response: json!({
                "message": "Unable to commit changes to database"
            }),
        },
    }
}
#[get("/pwordReset?<email>")]
pub fn request_password_reset(email: String, mut connection: PGConnection) -> StandardResponse {
    let mut transaction = transaction!(connection);

    let user = match User::from_email(email.clone(), &mut transaction) {
        Some(user) => user,
        None => {
            return StandardResponse {
                status: Status::BadRequest,
                response: json!({ "message": format!("No user found with email: {}", &email) }),
            }
        }
    };

    if !user.verified {
        return StandardResponse {
            status: Status::PreconditionRequired,
            response: json!({
                "message": "Your email address has not yet been verified"
            }),
        };
    }

    let token = match generate_token(email.clone(), TokenType::PasswordReset) {
        Ok(token) => token,
        Err(_) => {
            return StandardResponse {
                status: Status::UnprocessableEntity,
                response: json!({
                    "message": "Could not generate reset token"
                }),
            }
        }
    };

    let token = match store_token(
        token,
        TokenType::PasswordReset,
        email.clone(),
        &mut transaction,
    ) {
        Ok(token) => token,
        Err(_) => {
            return StandardResponse {
                status: Status::UnprocessableEntity,
                response: json!({
                    "message": "Could not store reset token"
                }),
            }
        }
    };

    match transaction.commit() {
        Ok(_) => {
            send_email_using_file!(
                email.as_str(),
                "Password Reset Request",
                "src/emails/password_reset_request.html",
                "{}",
                token.as_str()
            );

            StandardResponse {
                status: Status::Ok,
                response: json!({ "message": format!("Reset email has been sent to {}", &email) }),
            }
        }
        Err(_) => StandardResponse {
            status: Status::ServiceUnavailable,
            response: json!({
                "message": "Unable to commit changes to database"
            }),
        },
    }
}

#[post("/pwordReset")]
pub fn reset_password(auth: BasicAuth, mut connection: PGConnection) -> StandardResponse {
    let mut transaction = transaction!(connection);

    let token = match retrieve_token(auth.username, &mut transaction) {
        Some(token) => token,
        None => {
            return StandardResponse {
                status: Status::BadRequest,
                response: json!({
                    "message": "Token has expired or has already been used"
                }),
            }
        }
    };

    let email = match validate_token(token, TokenType::PasswordReset) {
        Ok(email) => email,
        Err(_) => {
            return StandardResponse {
                status: Status::BadRequest,
                response: json!({
                    "message": "Unable to verify reset token"
                }),
            }
        }
    };

    let user = match User::from_email(email.clone(), &mut transaction) {
        Some(user) => user,
        None => {
            return StandardResponse {
                status: Status::BadRequest,
                response: json!({
                    "message": format!("Could not find user with email: {}", email)
                }),
            }
        }
    };

    if auth.password.is_none() {
        return StandardResponse {
            status: Status::BadRequest,
            response: json!({
                "message": "Please provide a new valid password"
            }),
        };
    }

    if user
        .reset_password(auth.password.unwrap(), &mut transaction)
        .is_err()
    {
        return StandardResponse {
            status: Status::UnprocessableEntity,
            response: json!({
                "message": "Could not reset user's password"
            }),
        };
    }

    match transaction.commit() {
        Ok(_) => {
            send_email_using_file!(
                user.email.as_str(),
                "Your Elevate Password was Reset",
                "src/emails/password_reset_confirmation.html",
                "Your password was reset"
            );

            StandardResponse {
                status: Status::Ok,
                response: json!({
                    "message": "Successfully reset user's password",
                    "user": user
                }),
            }
        }

        Err(_) => StandardResponse {
            status: Status::ServiceUnavailable,
            response: json!({
                "message": "Unable to commit changes to database"
            }),
        },
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceToken {
    pub token: String,
}

#[put("/deviceTokens", format = "application/json", data = "<device_token>")]
pub fn update_device_token(
    device_token: Json<DeviceToken>,
    token: BearerToken,
    mut connection: PGConnection,
) -> StandardResponse {
    let mut transaction = transaction!(connection);
    let user = fetch_user!(token.token, TokenType::Auth, &mut transaction);

    if user
        .update_device_token(device_token.into_inner().token, &mut transaction)
        .is_err()
    {
        return StandardResponse {
            status: Status::UnprocessableEntity,
            response: json!({
                "message": "Device token cannot be updated at this time"
            }),
        };
    };

    match transaction.commit() {
        Ok(_) => StandardResponse {
            status: Status::Ok,
            response: json!({
                "message": "Device token updated successfully"
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
