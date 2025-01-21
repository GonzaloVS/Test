mod handlers;
mod tracing_config;

use std::{io};
use actix_web::{web, App, HttpServer};
use actix_web::middleware::{Compress};
use actix_files as af;
use tracing_subscriber;
use crate::tracing_config::init_tracing;

#[tokio::main]
async fn main() -> io::Result<()> {

    // Inicializar tracing
    init_tracing();

    HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(handlers::index_page)) // Ruta de la página de inicio
            .route("/analyze", web::get().to(handlers::analyze_logs)) // Ruta para ver los logs
            // Configura un servicio predeterminado para rutas no encontradas
            .default_service(web::route().to(handlers::custom_404))

            // Activa la compresión.
            // Solo si en el encabezado incluye Accept-Encoding o especifica un algoritmo compatible
            .wrap(Compress::default())
            .service(af::Files::new("/static", "./static").show_files_listing())
    })
        .bind("127.0.0.1:80")?
        .run()
        .await
}
