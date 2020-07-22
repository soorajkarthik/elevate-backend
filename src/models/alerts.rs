use chrono::NaiveDateTime;
use postgres::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertType {
    pub name: String,

    #[serde(rename = "alertLevel")]
    pub alert_level: i16,

    #[serde(rename = "createdAt")]
    #[serde(skip_deserializing)]
    pub created_at: Option<NaiveDateTime>,

    #[serde(rename = "updatedAt")]
    #[serde(skip_deserializing)]
    pub updated_at: Option<NaiveDateTime>,
}

#[macro_export]
macro_rules! alert_type {
    ($row:expr) => {
        AlertType {
            name: $row.get("name"),
            alert_level: $row.get("alert_level"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    };
}

impl AlertType {
    pub fn get_all(transaction: &mut Transaction) -> Vec<Self> {
        match transaction.query(
            "select * from alert_types
            ",
            &[],
        ) {
            Ok(rows) => rows
                .iter()
                .map(|row| alert_type!(row))
                .collect::<Vec<AlertType>>(),
            Err(err) => {
                error!("{}", err);
                Vec::new()
            }
        }
    }

    pub fn get_by_name(name: &String, transaction: &mut Transaction) -> Option<Self> {
        match transaction.query_one(
            "select * from alert_types where name = $1
            ",
            &[name],
        ) {
            Ok(row) => Some(alert_type!(row)),
            Err(err) => {
                error!("{}", err);
                None
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Alert {
    #[serde(skip_deserializing)]
    pub id: i64,

    #[serde(rename = "alertType")]
    #[serde(skip_serializing)]
    pub alert_type: String,

    #[serde(rename = "alertType")]
    #[serde(skip_deserializing)]
    pub alert_type_obj: Option<AlertType>,

    pub description: Option<String>,

    #[serde(skip_deserializing)]
    pub place: String,

    pub latitude: f32,

    pub longitude: f32,

    #[serde(rename = "createdBy")]
    pub created_by: String,

    #[serde(rename = "isResolved")]
    pub is_resolved: bool,

    #[serde(rename = "createdAt")]
    #[serde(skip_deserializing)]
    pub created_at: Option<NaiveDateTime>,

    #[serde(rename = "updatedAt")]
    #[serde(skip_deserializing)]
    pub updated_at: Option<NaiveDateTime>,
}

#[macro_export]
macro_rules! alert {
    ($row:expr) => {
        Alert {
            id: $row.get("id"),
            alert_type: $row.get("alert_type"),
            alert_type_obj: None,
            description: $row.get("description"),
            place: $row.get("place"),
            latitude: $row.get("latitude"),
            longitude: $row.get("longitude"),
            created_by: $row.get("created_by"),
            is_resolved: $row.get("is_resolved"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    };
}

#[macro_export]
macro_rules! alert_filled {
    ($row:expr) => {
        let mut alert = alert!(row);
        alert.fill(transaction);
        Ok(alert)
    };
}

impl Alert {
    pub fn init(&self, transaction: &mut Transaction) -> Result<Self, String> {
        match transaction.query_one(
            "insert into alerts (
                alert_type,
                description,
                place,
                latitude,
                longitude,
                created_by
            ) values ($1, $2, $3, $4, $5, $6)
            returning *
            ",
            &[
                &self.alert_type,
                &self.description,
                &self.place,
                &self.latitude,
                &self.longitude,
                &self.created_by,
            ],
        ) {
            Ok(row) => {
                let mut alert = alert!(row);
                alert.fill(transaction);
                Ok(alert)
            }
            Err(err) => {
                error!("{}", err);
                Err(String::from("Could not initialize alert"))
            }
        }
    }

    pub fn delete(&self, transaction: &mut Transaction) -> Result<Self, String> {
        match transaction.query_one(
            "delete from alerts where id = $1
            ",
            &[&self.id],
        ) {
            Ok(row) => {
                let mut alert = alert!(row);
                alert.fill(transaction);
                Ok(alert)
            }
            Err(err) => {
                error!("{}", err);
                Err(String::from("Could not delete alert"))
            }
        }
    }

    pub fn update(&self, transaction: &mut Transaction) -> Result<Self, String> {
        match transaction.query_one(
            "update alerts set
                alert_type = $1,
                description = $2,
                place = $3, 
                latitude = $4, 
                longitude = $5
            where id = $6 
            returning *
            ",
            &[
                &self.alert_type,
                &self.description,
                &self.place,
                &self.latitude,
                &self.longitude,
                &self.id,
            ],
        ) {
            Ok(row) => {
                let mut alert = alert!(row);
                alert.fill(transaction);
                Ok(alert)
            }
            Err(err) => {
                error!("{}", err);
                Err(String::from("Could not delete alert"))
            }
        }
    }

    pub fn fill(&mut self, transaction: &mut Transaction) {
        self.alert_type_obj = AlertType::get_by_name(&self.alert_type, transaction);
    }
}
