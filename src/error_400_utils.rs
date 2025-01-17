use actix_web::{HttpResponse};

pub fn handle_400_error() -> HttpResponse {
    HttpResponse::BadRequest()
        .append_header(("Content-Type", "application/json"))
        .body(r#"{"error": "Bad Request"}"#)
}

pub fn handle_401_error() -> HttpResponse {
    HttpResponse::Unauthorized()
        .append_header(("Content-Type", "application/json"))
        .body(r#"{"error": "Unauthorized"}"#)
}