use std::fs;
use actix_web::{HttpResponse};
use std::sync::{Arc, Mutex};
use tracing::{error, info, instrument};

lazy_static::lazy_static! {
    static ref LOGS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
}


pub(crate) async fn index_page() ->HttpResponse {
    let path ="./static/index/index.html";
    match fs::read(path) {
        Ok(file_content) => {
            let mime_type = mime_guess::from_path(path).first_or_octet_stream();
            HttpResponse::Ok()
                .content_type(mime_type.as_ref())
                .body(file_content)
        }
        Err(err) => {
            error!("Error al cargar el archivo {}: {:?}", path, err);
            custom_404().await
        }
    }

}

#[instrument] // También registra esta ruta en OpenTelemetry
pub(crate) async fn analyze_logs() -> HttpResponse {
    info!("Página de análisis de logs");
    HttpResponse::Ok().body("Consulta los datos en Jaeger o Grafana.")
}

pub(crate) async fn custom_404() -> HttpResponse {

    error!("Página no encontrada");

    match fs::read("./static/404/404.html") {
        Ok(content) => {
            info!("Mostrando página 404 personalizada");
            HttpResponse::NotFound()
                .content_type("text/html")
                .body(content)
        }
        Err(err) => {
            error!("Error al cargar la página 404: {:?}", err);
            HttpResponse::NotFound().body("404 - Página no encontrada")
        }
    }
}
