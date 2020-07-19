use std::str::FromStr;

use rocket::http::{ContentType, Status};
use rocket::http::hyper::header::Basic;
use rocket::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::{self, Responder, Response};
use rocket_contrib::json::JsonValue;

use crate::models::auth::{BasicAuth, BearerToken};
use crate::models::database::PGConnection;

pub struct StandardResponse {
    pub status: Status,
    pub response: JsonValue,
}

impl<'r> Responder<'r> for StandardResponse {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        Response::build_from(self.response.respond_to(request).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

/// Health checker
#[get("/ping")]
pub fn get_health() -> StandardResponse {
    StandardResponse {
        status: Status::Ok,
        response: json!("Server is alive!"),
    }
}

#[derive(Debug)]
pub enum FromRequestError {
    InvalidToken,
    MissingToken,
    UnableToConnect,
}

impl<'a, 'r> FromRequest<'a, 'r> for BasicAuth {
    type Error = FromRequestError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let headers = request.headers();

        if headers.contains("Authorization") {
            let mut auth_list = headers.get("Authorization");

            if let Some(token) = auth_list.next() {
                let mut split_token = token.split_whitespace();

                split_token.next();
                match split_token.next() {
                    Some(token_value) => {
                        let basic_header = match Basic::from_str(token_value) {
                            Ok(header) => header,
                            Err(_) => {
                                return Outcome::Failure((
                                    Status::BadRequest,
                                    FromRequestError::InvalidToken,
                                ));
                            }
                        };

                        return Outcome::Success(BasicAuth {
                            username: basic_header.username,
                            password: basic_header.password,
                        });
                    }
                    None => {
                        return Outcome::Failure((
                            Status::BadRequest,
                            FromRequestError::InvalidToken,
                        ));
                    }
                }
            }
        }

        Outcome::Failure((Status::BadRequest, FromRequestError::MissingToken))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for BearerToken {
    type Error = FromRequestError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let headers = request.headers();

        if headers.contains("Authorization") {
            let mut auth_list = headers.get("Authorization");

            if let Some(token) = auth_list.next() {
                let mut split_token = token.split_whitespace();
                split_token.next();

                if let Some(value) = split_token.next() {
                    return Outcome::Success(BearerToken {
                        token: String::from(value),
                    });
                } else {
                    return Outcome::Failure((Status::BadRequest, FromRequestError::InvalidToken));
                }
            }
        }

        Outcome::Failure((Status::BadRequest, FromRequestError::MissingToken))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for PGConnection {
    type Error = FromRequestError;

    fn from_request(_: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        match PGConnection::connect() {
            Ok(connection) => Outcome::Success(connection),
            Err(err) => {
                error!("{}", err);
                Outcome::Failure((
                    Status::ServiceUnavailable,
                    FromRequestError::UnableToConnect,
                ))
            }
        }
    }
}
