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
extern crate rocket_cors;
#[macro_use]
extern crate rocket_include_static_resources;
extern crate serde_json;

mod models;
mod services;
mod views;

use rocket_cors::{catch_all_options_routes, Cors, CorsOptions};
use rocket_include_static_resources::StaticResponse;

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

fn setup_cors() -> Result<Cors, rocket_cors::Error> {
    CorsOptions::default().to_cors() // Using default cors. Restrict access later
}

fn main() {
    dotenv::dotenv().ok();
    setup_logger().expect("Couldn't set up logger");
    let cors = setup_cors().expect("Couldn't generate CORS");
    rocket::ignite()
        .register(catchers![
            views::catchers::internal_error,
            views::catchers::not_found,
            views::catchers::service_error,
            views::catchers::unprocessable_entity,
            views::catchers::unauthorized,
            views::catchers::bad_request,
        ])
        .mount(
            "/",
            routes![
                views::request::get_health,
                views::assets::favicon,
                views::assets::favicon_png,
                views::assets::banner
            ],
        )
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
        .mount(
            "/pages",
            routes![
                views::pages::load_email_verification_page,
                views::pages::load_email_verification_success_page,
                views::pages::load_password_reset_page,
                views::pages::load_password_reset_success_page
            ],
        )
        .mount("/", catch_all_options_routes())
        .manage(cors.clone())
        .attach(cors)
        .attach(StaticResponse::fairing(|resources| {
            static_resources_initialize!(
                resources,
                //Image resources
                "favicon",
                "src/assets/icon.ico",
                "favicon-png",
                "src/assets/icon.png",
                "banner",
                "src/assets/banner.png",
                //Page resources
                "email_verification",
                "src/pages/email_verification.html",
                "email_verification_success",
                "src/pages/email_verification_success.html",
                "password_reset",
                "src/pages/password_reset.html",
                "password_reset_success",
                "src/pages/password_reset_success.html"
            );
        }))
        .launch();
}
