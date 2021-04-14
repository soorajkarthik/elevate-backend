use bcrypt::{hash, DEFAULT_COST};
use chrono::NaiveDateTime;
use postgres::Transaction;
use serde::{Deserialize, Serialize};

use crate::location;
use crate::models::auth::{validate_token, TokenType};
use crate::models::location::Location;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(skip_deserializing)]
    pub id: i64,

    pub name: String,

    pub email: String,

    #[serde(skip_serializing)]
    pub password: String,

    pub phone: Option<String>,

    #[serde(skip_deserializing)]
    pub verified: bool,

    #[serde(skip_deserializing)]
    #[serde(rename = "createdAt")]
    pub created_at: Option<NaiveDateTime>,

    #[serde(skip_deserializing)]
    #[serde(rename = "udpatedAt")]
    pub updated_at: Option<NaiveDateTime>,
}

#[macro_export]
macro_rules! user {
    ($row:expr) => {
        User {
            id: $row.get("id"),
            name: $row.get("name"),
            email: $row.get("email"),
            password: $row.get("password"),
            phone: $row.get("phone"),
            verified: $row.get("verified"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    };
}

#[macro_export]
macro_rules! fetch_user {
    ($token:expr, $token_type:expr, $transaction:expr) => {
        match User::from_token($token, $token_type, $transaction) {
            Some(user) => user,
            None => return StandardResponse {
                status: Status::BadRequest,
                response: json!({
                    "message": "User not found or token is invalid"
                })
            }
        }
    };
}

impl User {
    pub fn init(&self, transaction: &mut Transaction) -> Result<Self, String> {
        let password_hash = match hash(self.password.as_str(), DEFAULT_COST) {
            Ok(hash) => hash,
            Err(err) => return Err(err.to_string()),
        };

        match transaction.query_one(
            "insert into users (
                name,
                email,
                password,
                phone
            ) values ($1, $2, $3, $4)
            on conflict (email) do nothing 
            returning *
            ",
            &[&self.name, &self.email, &password_hash, &self.phone],
        ) {
            Ok(row) => Ok(user!(row)),
            Err(err) => {
                error!("{}", err);
                Err(String::from("Couldn't create new user"))
            }
        }
    }

    pub fn from_token(
        token: String,
        token_type: TokenType,
        transaction: &mut Transaction,
    ) -> Option<Self> {
        let user_email = match validate_token(token, token_type) {
            Ok(data) => data,
            Err(_) => return None,
        };

        Self::from_email(user_email, transaction)
    }

    pub fn from_email(email: String, transaction: &mut Transaction) -> Option<Self> {
        match transaction.query_one(
            "select * from users where email = $1
            ",
            &[&email],
        ) {
            Ok(row) => Some(user!(row)),
            Err(err) => {
                error!("{}", err);
                None
            }
        }
    }

    pub fn from_id(id: i64, transaction: &mut Transaction) -> Option<Self> {
        match transaction.query_one(
            "select * from users where id = $1
            ",
            &[&id],
        ) {
            Ok(row) => Some(user!(row)),
            Err(err) => {
                error!("{}", err);
                None
            }
        }
    }

    pub fn verify_email(&self, transaction: &mut Transaction) -> Result<Self, String> {
        match transaction.query_one(
            "update users set
                verified = true,
                updated_at = now()
            where id = $1
            returning *
            ",
            &[&self.id],
        ) {
            Ok(row) => Ok(user!(row)),
            Err(err) => {
                error!("{}", err);
                Err(String::from("Could not update user verification status"))
            }
        }
    }

    pub fn get_location(&self, transaction: &mut Transaction) -> Option<Location> {
        match transaction.query_one(
            "select * from locations where user_id = $1
            ",
            &[&self.id],
        ) {
            Ok(row) => Some(location!(row)),
            Err(err) => {
                error!("{}", err);
                None
            }
        }
    }

    pub fn reset_password(
        &self,
        password: String,
        transaction: &mut Transaction,
    ) -> Result<Self, String> {
        let password_hash = match hash(password.as_str(), DEFAULT_COST) {
            Ok(hash) => hash,
            Err(err) => return Err(err.to_string()),
        };

        match transaction.query_one(
            "update users set password = $1 where id = $2
            returning *
            ",
            &[&password_hash, &self.id],
        ) {
            Ok(row) => Ok(user!(row)),
            Err(err) => {
                error!("{}", err);
                Err(String::from("Could not reset user's password"))
            }
        }
    }

    pub fn update_device_token(
        &self,
        device_token: String,
        transaction: &mut Transaction,
    ) -> Result<(), String> {
        match transaction.query_one(
            "insert into firebase_device_tokens (
                user_id, 
                token,
                updated_at
            ) values ($1, $2, now())
            on conflict (user_id) do update set 
                token = excluded.token,
                updated_at = now()
            returning *
        ",
            &[&self.id, &device_token],
        ) {
            Ok(_) => return Ok(()),
            Err(err) => {
                error!("{}", err);
                return Err(String::from("Error while updating token"));
            }
        }
    }
}
