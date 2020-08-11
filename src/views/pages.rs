//
// Needed response types:
// image/jpg
// image/webp
// text/html
//
// Needed requests:
//
// get logo
// get favicon.ico
// email verification page (get token from url)
// password reset page (get token from url)
//
use rocket_include_static_resources::StaticResponse;

#[get("/verify?<token>")]
pub fn load_email_verification_page(token: String) -> StaticResponse {
    static_response!("email_verification")
}

#[get("/verify/success")]
pub fn load_email_verification_success_page() -> StaticResponse {
    static_response!("email_verification_success")
}

#[get("/pwordReset?<token>")]
pub fn load_password_reset_page(token: String) -> StaticResponse {
    static_response!("password_reset")
}

#[get("/pwordReset/success")]
pub fn load_password_reset_success_page() -> StaticResponse {
    static_response!("password_reset_success")
}
