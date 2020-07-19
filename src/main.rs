#![feature(proc_macro_hygiene, decl_macro)]
#![crate_name = "elevate_backend"]

extern crate bcrypt;
extern crate chrono;
extern crate dotenv;
extern crate jsonwebtoken;
extern crate lettre;
extern crate lettre_email;
#[macro_use]
extern crate log;
extern crate postgres;
extern crate postgres_types;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate serde_json;

use rocket::http::Status;

use crate::views::request::StandardResponse;

mod models;
mod views;

/// Health checker
#[get("/ping")]
pub fn get_health() -> StandardResponse {
    StandardResponse {
        status: Status::Ok,
        response: json!("Server is alive!"),
    }
}

fn main() {
    dotenv::dotenv().ok();
    rocket::ignite()
        .mount("/", routes![
            get_health
        ])
        .mount("/users", routes![
            views::user::login,
            views::user::create_user,
            views::user::verify_email
        ])
        .launch();
}
