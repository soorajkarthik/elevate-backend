use postgres::{Client, NoTls};
use std::env;
use std::ops::{Deref, DerefMut};

pub struct PGConnection {
    pub client: Client
}

impl Deref for PGConnection {
    type Target = Client;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for PGConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

impl PGConnection {

    pub fn connect () -> Result<Self, String> {
        let connection_str = format!(
            "{}://{}:{}@{}/{}",
            env::var("DB_USER").unwrap(),
            env::var("DB_PASSWORD").unwrap(),
            env::var("DB_HOST").unwrap(),
            env::var("DB_PORT").unwrap(),
            env::var("DB_DATABASE").unwrap()
        );
        match Client::connect(connection_str.as_str(), NoTls) {
            Ok(client) => Ok(PGConnection {client}),
            Err(err) => {
                eprintln!("{}", err);
                Err(String::from("Could not connect to postgres database"))
            }
        }
    }
}

#[macro_export]
macro_rules! transaction {
    ($connection:expr) => {
        match (&mut $connection).transaction() {
            Ok(transaction) => transaction,
            Err(_) => {
                eprintln!("Failed to get database transaction.");
                return StandardResponse {
                    status_code: Status::ServiceUnavailable,
                    json: json!({
                        "message": "The server is currently unavailable."
                    })
                };
            }
        }
    };
}