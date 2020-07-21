use std::env;
use std::ops::Deref;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, Header, Validation};
use postgres::Transaction;
use serde::{Deserialize, Serialize};

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
    Verification,
}

#[macro_export]
macro_rules! secret {
    ($token_type:expr) => {
        match $token_type {
            TokenType::Auth => env::var("AUTH_SECRET").unwrap(),
            TokenType::Verification => env::var("VERIFICATION_SECRET").unwrap(),
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
            error!("{}", err.to_string());
            return Err(String::from("Could not sign authentication token."));
        }
    }
}

pub fn validate_token(token: String, token_type: TokenType) -> Result<String, String> {
    let secret = secret!(token_type);

    match decode::<Claims>(
        token.as_str(),
        secret.as_str().as_ref(),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(verified_token) => Ok(verified_token.claims.sub),
        Err(err) => Err(err.to_string()),
    }
}

pub fn store_token(
    token: String,
    token_type: TokenType,
    created_for: String,
    transaction: &mut Transaction,
) -> Result<String, String> {
    match transaction.query(
        "delete from tokens where created_at < now() - interval '7 days'
        ",
        &[],
    ) {
        Ok(rows) => rows
            .iter()
            .for_each(|row| info!("Deleted expired token {}", row.get::<usize, String>(0))),
        Err(err) => error!("Error while deleting expired tokens: {}", err),
    }

    let token_type_text = match token_type {
        TokenType::Verification => String::from("email verification"),
        _ => String::from("other"), // Should never happen
    };

    match transaction.query_one(
        "insert into tokens (
            token,
            token_type,
            created_for
        ) values ($1, $2, $3)
        returning token
        ",
        &[&token, &token_type_text, &created_for],
    ) {
        Ok(row) => Ok(row.get(0)),
        Err(err) => {
            error!("{}", err);
            Err(String::from(""))
        }
    }
}

pub fn retrieve_token(token: String, transaction: &mut Transaction) -> Option<String> {
    match transaction.query_one(
        "delete from tokens
            where token = $1
            and created_at > now() - interval '1 day'
        returning token
        ",
        &[&token],
    ) {
        Ok(row) => Some(row.get(0)),
        Err(err) => {
            error!("{}", err);
            None
        }
    }
}
