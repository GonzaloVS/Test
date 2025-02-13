use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::format;
pub async fn api_handler(path: web::Path<String>) -> impl Responder {
    let tail = path.into_inner();
    HttpResponse::Ok().json(json!({
        "message": format!("Este es un archivo JSON para la ruta /api/{}", tail)
    }))
}
