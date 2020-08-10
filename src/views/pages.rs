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

#[get("/favicon.ico")]
pub fn favicon() -> StaticResponse {
    static_response!("favicon")
}

#[get("/favicon-16.png")]
pub fn favicon_png() -> StaticResponse {
    static_response!("favicon-png")
}

#[get("/banner")]
pub fn banner() -> StaticResponse {
    static_response!("banner")
}
