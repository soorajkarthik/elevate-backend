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

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDistanceWrapper {
    pub firebase_device_token: String,
    pub distance: f32,
}

#[macro_export]
macro_rules! token_distance_wrapper {
    ($row:expr) => {
        TokenDistanceWrapper {
            firebase_device_token: $row.get("token"),
            distance: $row.get("calculate_distance"),
        }
    };
}

pub const LAT_LNG_VIEW_PORT: f32 = 0.145; // 0.145 degrees ~ 10 miles

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
                alert.populate(transaction);
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
                alert.populate(transaction);
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
                longitude = $5,
                updated_at = now()
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
                alert.populate(transaction);
                Ok(alert)
            }
            Err(err) => {
                error!("{}", err);
                Err(String::from("Could not delete alert"))
            }
        }
    }

    pub fn get_by_id(id: i64, transaction: &mut Transaction) -> Option<Alert> {
        match transaction.query_one(
            "select * from alerts where id = $1
            ",
            &[&id],
        ) {
            Ok(row) => {
                let mut alert = alert!(row);
                alert.populate(transaction);
                Some(alert)
            }
            Err(err) => {
                error!("{}", err);
                None
            }
        }
    }

    pub fn resolve(&self, transaction: &mut Transaction) -> Result<Self, String> {
        match transaction.query_one(
            "update alerts set 
                is_resolved = true,
                updated_at = now()
            where id = $1
            ",
            &[&self.id],
        ) {
            Ok(row) => Ok(alert!(row)),
            Err(err) => {
                error!("{}", err);
                Err(String::from("Could not mark alert as resolved"))
            }
        }
    }

    pub fn populate(&mut self, transaction: &mut Transaction) {
        self.alert_type_obj = AlertType::get_by_name(&self.alert_type, transaction);
    }

    pub fn near_by_user_tokens(&self, transaction: &mut Transaction) -> Vec<TokenDistanceWrapper> {
        match transaction.query(
            "select fdt.token, calculate_distance($1, $2, l.latitude, l.longitude) from firebase_device_tokens fdt
            inner join locations l
                on  fdt.user_id = l.user_id
            where 
                l.latitude > $1 - $3
                and l.latitude < $1 + $3
                and l.longitude > $2 - $3
                and l.longitude < $2 + $3
            ", &[&self.latitude, &self.longitude, &LAT_LNG_VIEW_PORT]
        ) {
            Ok(rows) => rows.iter().map(|row| token_distance_wrapper!(row)).collect::<Vec<TokenDistanceWrapper>>(),
            Err(err) => {
                error!("{}", err);
                Vec::new()
            }
        }
    }
}
