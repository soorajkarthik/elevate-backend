#![feature(proc_macro_hygiene, decl_macro)]
#![crate_name = "elevate_backend"]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

extern crate bcrypt;
extern crate chrono;
extern crate dotenv;
extern crate jsonwebtoken;
extern crate lettre;
extern crate lettre_email;
extern crate postgres;
extern crate postgres_types;
extern crate serde_json;

mod models;
mod views;

/// Health checker
#[get("/ping")]
pub fn get_health() -> String {
    String::from("Server is alive!")
}

fn main() {
    dotenv::dotenv().ok();
    rocket::ignite().mount("/", routes![get_health,]).launch();
}
