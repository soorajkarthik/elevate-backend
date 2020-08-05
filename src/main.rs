#![feature(proc_macro_hygiene, decl_macro)]
#![crate_name = "elevate_backend"]

extern crate bcrypt;
extern crate chrono;
extern crate dotenv;
extern crate fern;
extern crate jsonwebtoken;
extern crate lettre;
extern crate lettre_email;
#[macro_use]
extern crate log;
extern crate postgres;
extern crate postgres_types;
extern crate reqwest;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate serde_json;

mod models;
mod services;
mod views;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Utc::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

fn main() {
    dotenv::dotenv().ok();
    setup_logger().expect("Couldn't set up logger");
    rocket::ignite()
        .register(catchers![
            views::catchers::internal_error,
            views::catchers::not_found,
            views::catchers::service_error,
            views::catchers::unprocessable_entity,
            views::catchers::unauthorized,
            views::catchers::bad_request,
        ])
        .mount("/", routes![views::request::get_health])
        .mount(
            "/users",
            routes![
                views::user::login,
                views::user::create_user,
                views::user::verify_email,
                views::user::send_verification_email,
                views::user::request_password_reset,
                views::user::reset_password,
                views::location::update_user_location,
                views::location::get_location
            ],
        )
        .mount(
            "/alerts",
            routes![
                views::alert::get_alert_types,
                views::alert::create_alert,
                views::alert::update_alert,
                views::alert::resolve_alert,
                views::alert::delete_alert,
                views::alert::get_by_viewport
            ],
        )
        .launch();
}
