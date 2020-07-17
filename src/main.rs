#![feature(proc_macro_hygiene, decl_macro)]
#![crate_name = "elevate_backend"]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

extern crate bcrypt;
extern crate chrono;
extern crate dotenv;
extern crate postgres;
extern crate postgres_types;
extern crate serde_json;
extern crate lettre;
extern crate lettre_email;

/// Health checker
#[get("/")]
pub fn get_health() -> String {
    String::from("hi")
}

/// Respond Flow dataserver launch point.
fn main() {
    dotenv::dotenv().ok();
    rocket::ignite()
        .mount("/", routes![
            get_health,
        ])
        .launch();
}
