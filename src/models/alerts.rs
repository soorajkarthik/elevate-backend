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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_type_obj: Option<AlertType>,

    pub description: Option<String>,

    #[serde(skip_deserializing)]
    pub place: String,

    pub latitude: f32,

    pub longitude: f32,

    #[serde(rename = "createdBy")]
    pub created_by: String,

    #[serde(rename = "createdAt")]
    #[serde(skip_deserializing)]
    pub created_at: Option<NaiveDateTime>,

    #[serde(rename = "updatedAt")]
    #[serde(skip_deserializing)]
    pub updated_at: Option<NaiveDateTime>,
}