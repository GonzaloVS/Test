use actix_web::{web, App, HttpResponse, HttpServer, Responder};

// async fn api_handler() -> impl Responder {
//     HttpResponse::Ok().json(serde_json::json!({
//         "message": "Este es un archivo JSON desde el servidor Rust"
//     }))
// }

async fn api_handler(path: web::Path<String>) -> impl Responder {
    let tail = path.into_inner();
    HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Este es un archivo JSON para la ruta /api/{}", tail)
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            //.route("/api/{tail:.*}", web::get().to(api_handler))
            .route("/api/{_:.*}", web::get().to(api_handler))
    })
        //.bind("127.0.0.1:8000")?
        .bind("0.0.0.0:8000")? // Escucha en todas las interfaces IPv4 e IPv6 (por caddy)
        .run()
        .await
}
