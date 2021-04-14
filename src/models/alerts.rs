use crate::models::user::User;
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
pub struct AlertUserInfo {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

#[macro_export]
macro_rules! alert_user_info {
    ($alert:expr, $user:expr) => {
        AlertUserInfo {
            name: $user.name,
            email: if $alert.display_email {
                Some($user.email)
            } else {
                Option::None
            },
            phone: if $alert.display_phone {
                $user.phone
            } else {
                Option::None
            },
        }
    };
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

    #[serde(rename = "displayEmail")]
    pub display_email: bool,

    #[serde(rename = "displayPhone")]
    pub display_phone: bool,

    #[serde(rename = "trackLocation")]
    pub track_location: bool,

    #[serde(skip)]
    pub created_by: String,

    #[serde(rename = "userInfo")]
    #[serde(skip_deserializing)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_info: Option<AlertUserInfo>,

    #[serde(rename = "isResolved")]
    #[serde(skip_deserializing)]
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
            display_email: $row.get("display_email"),
            display_phone: $row.get("display_phone"),
            track_location: $row.get("track_location"),
            created_by: $row.get("created_by"),
            user_info: Option::None,
            is_resolved: $row.get("is_resolved"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertNotificationInfo {
    pub distance: f32,
    pub token: String,
}

#[macro_export]
macro_rules! alert_notification_info {
    ($row:expr) => {
        AlertNotificationInfo {
            distance: $row.get("distance"),
            token: $row.get("token"),
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
                display_email,
                display_phone,
                track_location,
                created_by
            ) values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            returning *
            ",
            &[
                &self.alert_type,
                &self.description,
                &self.place,
                &self.latitude,
                &self.longitude,
                &self.display_email,
                &self.display_phone,
                &self.track_location,
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
            returning *
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
                display_email = $6,
                display_phone = $7,
                track_location = $8,
                updated_at = now()
            where id = $9 
            returning *
            ",
            &[
                &self.alert_type,
                &self.description,
                &self.place,
                &self.latitude,
                &self.longitude,
                &self.display_email,
                &self.display_phone,
                &self.track_location,
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
            returning *
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
        if let Some(user) = User::from_email(String::from(&self.created_by), transaction) {
            self.user_info = Some(alert_user_info!(self, user));
        }
    }

    pub fn get_notification_info(
        &self,
        transaction: &mut Transaction,
    ) -> Vec<AlertNotificationInfo> {
        match transaction.query(
            "select 
                calculate_distance($1, $2, l.latitude, l.longitude) as distance,
                fdt.token 
            from firebase_device_tokens fdt
            inner join locations l
                on  fdt.user_id = l.user_id
            where 
                l.latitude > $1::real - $3::real
                and l.latitude < $1::real + $3::real
                and l.longitude > $2::real - $3::real
                and l.longitude < $2::real + $3::real
            ",
            &[&self.latitude, &self.longitude, &LAT_LNG_VIEW_PORT],
        ) {
            Ok(rows) => rows
                .iter()
                .map(|row| alert_notification_info!(row))
                .collect::<Vec<AlertNotificationInfo>>(),
            Err(err) => {
                error!("{}", err);
                Vec::new()
            }
        }
    }

    pub fn get_by_viewport(
        ne_lat: f32,
        ne_lng: f32,
        sw_lat: f32,
        sw_lng: f32,
        transaction: &mut Transaction,
    ) -> Vec<Self> {
        match transaction.query(
            "select * from alerts where 
                latitude < $1
                and longitude < $2
                and latitude > $3
                and longitude > $4
            ",
            &[&ne_lat, &ne_lng, &sw_lat, &sw_lng],
        ) {
            Ok(rows) => {
                let mut res = Vec::new();
                for row in rows {
                    let mut alert = alert!(row);
                    alert.populate(transaction);
                    res.push(alert);
                }
                res
            }
            Err(err) => {
                error!("{}", err);
                Vec::new()
            }
        }
    }
}
