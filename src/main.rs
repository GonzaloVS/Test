use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn api_handler() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Este es un archivo JSON desde el servidor Rust"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/api/{tail:.*}", web::get().to(api_handler))
    })
        .bind("127.0.0.1:8000")?
        .run()
        .await
}
