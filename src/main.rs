use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use std::path::Path;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index_page))
            .route("/login", web::get().to(login_page))
            .route("/login.js", web::get().to(login_script))
            .route("/all.css", web::get().to(allcss_page))
            .route("/antigua-url", web::get().to(redirect_301))
            .route("/temporal-url", web::get().to(redirect_302))
            .route("/api/{tail:.*}", web::to(api_handler)) // Genérico para /api/*
            .default_service(web::to(not_found)) // Manejo genérico de 404
    })
        .bind("127.0.0.1:8000")? // Puerto para el proxy
        .run()
        .await
}

async fn index_page() -> impl Responder {
    HttpResponse::Ok().body("Página de inicio")
}

async fn login_page() -> impl Responder {
    serve_static_file("./static/login/login.html")
}

async fn login_script() -> impl Responder {
    serve_static_file("./static/login/login_script.js")
}

async fn allcss_page() -> impl Responder {
    serve_static_file("./static/all.css")
}

async fn redirect_301() -> impl Responder {
    HttpResponse::MovedPermanently()
        .append_header(("Location", "/nuevo-destino"))
        .finish()
}

async fn redirect_302() -> impl Responder {
    HttpResponse::Found()
        .append_header(("Location", "/temporal-destino"))
        .finish()
}

async fn api_handler(req: HttpRequest) -> impl Responder {
    let path = req.match_info().query("tail").unwrap_or_default();
    HttpResponse::Ok().body(format!("API alcanzada en: /api/{}", path))
}

async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("404 Página no encontrada")
}

fn serve_static_file(path: &str) -> HttpResponse {
    if Path::new(path).exists() {
        HttpResponse::Ok()
            .append_header(("Cache-Control", "max-age=31536000"))
            .body(std::fs::read_to_string(path).unwrap_or_else(|_| "Error al leer el archivo".to_string()))
    } else {
        HttpResponse::NotFound().body("Archivo no encontrado")
    }
}
