use crate::views::request::HTMLResponse;
use rocket::http::Status;
use rocket_include_static_resources::StaticResponse;
use std::fs::read_to_string;

#[get("/verify")]
pub fn load_email_verification_request_page() -> StaticResponse {
    static_response!("email_verification_request")
}

#[get("/verify?<token>")]
pub fn load_email_verification_page(token: String) -> HTMLResponse {
    match read_to_string("src/pages/email_verification.html") {
        Ok(page) => HTMLResponse {
            status: Status::Ok,
            template: Some(page.replace("{}", token.as_str())),
        },
        Err(_) => HTMLResponse {
            status: Status::NotFound,
            template: None,
        },
    }
}

#[get("/verify/success")]
pub fn load_email_verification_success_page() -> HTMLResponse {
    match read_to_string("src/pages/message_template.html") {
        Ok(page) => HTMLResponse {
            status: Status::Ok,
            template: Some(page.replace("{}", "Your email has successfully been verified. You should now be able to log in and start using our app. Thank you for joining Elevate!")),
        },
        Err(_) => HTMLResponse {
            status: Status::NotFound,
            template: None,
        },
    }
}

#[get("/pwordReset")]
pub fn load_password_reset_request_page() -> StaticResponse {
    static_response!("password_reset_request")
}

#[get("/pwordReset?<token>")]
pub fn load_password_reset_page(token: String) -> HTMLResponse {
    match read_to_string("src/pages/password_reset.html") {
        Ok(page) => HTMLResponse {
            status: Status::Ok,
            template: Some(page.replace("{}", token.as_str())),
        },
        Err(_) => HTMLResponse {
            status: Status::NotFound,
            template: None,
        },
    }
}

#[get("/pwordReset/success")]
pub fn load_password_reset_success_page() -> HTMLResponse {
    match read_to_string("src/pages/message_template.html") {
        Ok(page) => HTMLResponse {
            status: Status::Ok,
            template: Some(page.replace("{}", "Your password has successfully been reset. You should now be able to log in using the new password you just set. Thank you for using Elevate!")),
        },
        Err(_) => HTMLResponse {
            status: Status::NotFound,
            template: None,
        },
    }
}
