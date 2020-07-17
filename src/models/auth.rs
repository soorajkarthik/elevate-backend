use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::ops::Deref;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    iat: i64,
    exp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BearerToken {
    pub token: String,
}

impl Deref for BearerToken {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.token
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicAuth {
    pub username: String,
    pub password: Option<String>,
}

pub enum TokenType {
    Auth,
}

#[macro_export]
macro_rules! secret {
    ($token_type:expr) => {
        match $token_type {
            TokenType::Auth => env::var("AUTH_SECRET").unwrap(),
        }
    };
}

pub fn generate_token(identifier: String, token_type: TokenType) -> Result<String, String> {
    let secret = secret!(token_type);

    let now = Utc::now();
    let claims = Claims {
        exp: (now + Duration::days(90)).timestamp(),
        iat: now.timestamp(),
        sub: identifier,
    };

    match encode(
        &Header::new(Algorithm::HS256),
        &claims,
        secret.as_str().as_ref(),
    ) {
        Ok(token) => Ok(token),
        Err(err) => {
            eprintln!("{}", err.to_string());
            return Err(String::from("Could not sign authentication token."));
        }
    }
}

pub fn validate_token(token: String, token_type: TokenType) -> Result<String, String> {
    let secret = secret!(token_type);

    match decode::<Claims>(
        &token,
        secret.as_str().as_ref(),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(verified_token) => Ok(verified_token.claims.sub),
        Err(err) => Err(err.to_string()),
    }
}
