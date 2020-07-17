use postgres::Transaction;
use serde::{Serialize, Deserialize};
use chrono::NaiveDateTime;
use bcrypt::{hash, DEFAULT_COST};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {

    #[serde(skip_deserializing)]
    pub id: i64,

    pub name: String,

    pub email: String,

    #[serde(skip_serializing)]
    pub password: String,

    pub phone: Option<String>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub longitude: Option<f32>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub latitude: Option<f32>,

    #[serde(rename = "acceptedLocationTracking")]
    pub accepted_location_tracking: bool,

    #[serde(skip_deserializing)]
    #[serde(rename = "createdAt")]
    pub created_at: Option<NaiveDateTime>,

    
    #[serde(skip_deserializing)]
    #[serde(rename = "udpatedAt")]
    pub updated_at: Option<NaiveDateTime>
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
            latitude: $row.get("latitude"),
            longitude: $row.get("longitude"),
            accepted_location_tracking: $row.get("accepted_location_tracking"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
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
                phone,
            ) values ($1, $2, $3, $4)
            on conflict (email) do nothing 
            returning *
            ", &[&self.name, &self.email, &password_hash, &self.phone]
        ) {
            Ok(row) => Ok(user!(row)),
            Err(err) => {
                error!("{}", err);
                Err(String::from("Couldn't create new user"))
            }
        }
    }

    pub fn update_location (&self, transaction: &mut Transaction) -> Result<Self, String> {

        match transaction.query_one(
            "update users set 
            latitude = $1,
            longitude = $2
            where id = $3
            returning *
            ", &[&self.latitude, &self.longitude, &self.id]
        ) {
            Ok(row) => Ok(user!(row)),
            Err(err) => {
                error!("{}", err);
                Err(String::from("Couldn't create new user"))
            }
        }
    }
}