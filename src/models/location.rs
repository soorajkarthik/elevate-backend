use chrono::NaiveDateTime;
use postgres::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    #[serde(rename = "userId")]
    pub user_id: i64,

    pub latitude: f32,

    pub longitude: f32,

    #[serde(rename = "createdAt")]
    #[serde(skip_deserializing)]
    pub created_at: Option<NaiveDateTime>,

    #[serde(rename = "updatedAt")]
    #[serde(skip_deserializing)]
    pub updated_at: Option<NaiveDateTime>,
}

#[macro_export]
macro_rules! location {
    ($row:expr) => {
        Location {
            user_id: $row.get("user_id"),
            latitude: $row.get("latitude"),
            longitude: $row.get("longitude"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at")
        }
    };
}

impl Location {
    pub fn init_or_update(&self, transaction: &mut Transaction) -> Result<Self, String> {
        match transaction.query_one(
            "insert into locations (
                user_id,
                latitude,
                longitude
            ) values ($1, $2, $3)
            on conflict (user_id) do update
            set
                latitude = excluded.latitude,
                longitude = excluded.longitude,
                updated_at = now()
            returning *
            ", &[&self.user_id, &self.latitude, &self.longitude],
        ) {
            Ok(row) => Ok(location!(row)),
            Err(err) => {
                error!("{}", err);
                Err(String::from("Couldn't update location information"))
            }
        }
    }
}