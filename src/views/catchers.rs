use rocket_contrib::json::JsonValue;

#[catch(500)]
pub fn internal_error() -> JsonValue {
    json!({
        "message": "The server cannot be contacted at this moment."
    })
}

#[catch(404)]
pub fn not_found() -> JsonValue {
    json!({
        "message": "The requested resource could not be found."
    })
}

#[catch(503)]
pub fn service_error() -> JsonValue {
    json!({
        "message": "The server is unavailable at this time."
    })
}

#[catch(422)]
pub fn unprocessable_entity() -> JsonValue {
    json!({
        "message": "Server could not process your request, are missing arguments?"
    })
}

#[catch(401)]
pub fn unauthorized() -> JsonValue {
    json!({
        "message": "Current user not authorized."
    })
}

#[catch(400)]
pub fn bad_request() -> JsonValue {
    json!({
        "message": "The request is missing parameters."
    })
}