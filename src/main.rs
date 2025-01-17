mod file_utils;
mod file_cache;
mod error_400_utils;
mod css_utils;
mod metrics;

use actix_cors::Cors;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::Key, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use governor::{state::InMemoryState, Quota, RateLimiter};
use governor::clock::DefaultClock;
use std::io;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use governor::state::NotKeyed;
use crate::metrics::{export_metrics, Metrics};


#[tokio::main]
async fn main() -> io::Result<()> {

    // Crear el registro de métricas y las métricas
    let registry = Arc::new(prometheus::Registry::new());
    let metrics = Arc::new(Metrics::new(registry.clone()));

    let css_dir = "./static";
    let output_file = "./static/all.css";

    //Primera combinación inicial
    if let Err(e) = css_utils::combine_css(css_dir, output_file).await {
        eprintln!("Error inicial al combinar CSS: {}", e);
    }

    // // Iniciar el monitoreo de cambios
    // tokio::spawn(async move {
    //     if let Err(e) = css_utils::monitor_changes(css_dir, output_file).await {
    //         eprintln!("Error en el monitoreo de cambios: {}", e);
    //     }
    // });

    //env_logger::init(); // Inicializa logs

    // Configuración de direcciones y puertos
    let http_addr = "127.0.0.1:80";
    //let https_addr = "127.0.0.1:443";

    // tokio::join!(
    //     start_http_server(http_addr),
    //     //start_https_server(https_addr)
    // );

    // Iniciar el servidor HTTP
    start_http_server(http_addr, metrics)
        .await
}

async fn start_http_server(addr: &str, metrics: Arc<Metrics>) -> io::Result<()> {
    HttpServer::new({
        let metrics = metrics.clone();
        move || app_factory(metrics.clone())
    })
        .bind(addr)?
        .workers(8)
        .max_connections(50_000)
        .max_connection_rate(1_000)
        .client_request_timeout(Duration::from_secs(30))
        .client_disconnect_timeout(Duration::from_secs(5))
        .run()
        .await
}

// async fn start_https_server(addr: &str,  metrics: Arc<Metrics>) -> io::Result<()> {
//     HttpServer::new(move || app_factory(metrics.clone()))
//         .bind_openssl(addr, load_ssl_keys()?)?
//         .workers(8)
//         .max_connections(50_000)
//         .max_connection_rate(1_000)
//         .client_request_timeout(std::time::Duration::from_secs(30))
//         .client_disconnect_timeout(std::time::Duration::from_secs(5))
//         .run()
//         .await
// }

// fn app_factory(metrics: Arc<Metrics>) -> App<impl ServiceFactory<ServiceRequest>>  {
//     let rate_limiter = create_rate_limiter();
//
//     App::new()
//     //     .wrap(
//     //         SessionMiddleware::new(
//     //             CookieSessionStore::default(),
//     //             secret_key(),
//     //         )
//     //     )
//     //     .cookie_same_site(actix_web::cookie::SameSite::Strict)
//
//     // )
//         .wrap(
//             SessionMiddleware::builder(
//                 CookieSessionStore::default(),
//                 secret_key(),
//             )
//                 .cookie_same_site(actix_web::cookie::SameSite::Strict)
//                 .build(),
//         )
//
//         .wrap(
//             Cors::default()
//                 .allowed_origin("https://example.com")
//                 .allowed_methods(vec!["GET", "POST"])
//                 .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE])
//                 .max_age(3600),
//         )
//         .wrap(
//             middleware::DefaultHeaders::new()
//                 .add(("X-Custom-Header", "Value"))
//                 .add(("Strict-Transport-Security", "max-age=63072000; includeSubDomains"))
//                 .add(("X-Frame-Options", "DENY"))
//                 .add(("X-Content-Type-Options", "nosniff")) // Corregido el error tipográfico
//                 .add(("Content-Security-Policy", "default-src 'self'; script-src 'self'")),
//         )
//         //.wrap(rate_limiter)
//         .wrap(middleware::Compress::default())
//         .app_data(web::Data::new(metrics.clone()))
fn app_factory(metrics: Arc<Metrics>) -> impl ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse, Error = actix_web::Error, InitError = ()> {
    App::new()
        .wrap(
            SessionMiddleware::builder(
                CookieSessionStore::default(),
                secret_key(),
            )
                .cookie_same_site(actix_web::cookie::SameSite::Strict)
                .build(),
        )
        .wrap(
            Cors::default()
                .allowed_origin("https://example.com")
                .allowed_methods(vec!["GET", "POST"])
                .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE])
                .max_age(3600),
        )
        .wrap(middleware::DefaultHeaders::new().add(("X-Example-Header", "Value")))
        .wrap(middleware::Compress::default())
        .app_data(web::Data::new(metrics.clone()))

        .route("/metrics", web::get().to(move || {
            let registry = metrics.registry.clone(); // Usa el registry compartido
            async move { export_metrics(registry).await }
        }))
        .route("/", web::get().to(index_page))
        .route("/index.js", web::get().to(index_script))
        .route("/login", web::get().to(login_page))
        .route("/login.js", web::get().to(login_script))
        .route("/all.css", web::get().to(allcss_page))
        .route("/antigua-url", web::get().to(redirect_301))
        .route("/temporal-url", web::get().to(redirect_302))
        .route("/items", web::get().to(items_handler))
        .route("/static/{filename:.*}", web::get().to(static_files))
        .default_service(web::route().to(not_found))
        .app_data(web::JsonConfig::default().error_handler(|err, _req| {
            actix_web::error::InternalError::from_response(
                err,
                error_400_utils::handle_400_error().into(),
            )
                .into()
        }))
}


fn create_rate_limiter() -> RateLimiter<NotKeyed, InMemoryState, DefaultClock> {
    // Crear una cuota: 100 solicitudes por cada 60 segundos
    let quota = Quota::per_minute(NonZeroU32::new(100).unwrap());

    // Crear un RateLimiter con memoria en el estado y reloj predeterminado
    RateLimiter::direct(quota)
}
// Función para cargar certificados SSL
// fn load_ssl_keys() -> std::io::Result<openssl::ssl::SslAcceptor> {
//     use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
//
//     let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
//     builder.set_private_key_file("certs/private.key", SslFiletype::PEM)?;
//     builder.set_certificate_chain_file("certs/certificate.crt")?;
//     Ok(builder.build())
// }




async fn static_files(req: HttpRequest) -> HttpResponse {
    let filename: String = req.match_info().query("filename").parse().unwrap();
    let path = format!("./static/{}", filename);

    println!("RUTA GENÉRICA: Solicitado: {}, Mapeado a: {}", filename, path);

    if Path::new(&path).exists() {
        file_cache::file_handler(&path)
    } else {
        HttpResponse::NotFound().body("Archivo no encontrado")
    }
}


async fn index_page(metrics: web::Data<Metrics>, session: Session) -> impl Responder {
    metrics.http_requests_total.inc(); // Incrementar contador de solicitudes
    let timer = metrics.request_duration.start_timer(); // Iniciar temporizador

    let response = if session.get::<String>("auth_token").unwrap_or(None).is_some() {
        file_cache::file_handler("./static/index/index.html")
    } else {
        HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish()
    };

    timer.observe_duration(); // Registrar duración de la solicitud
    response
}


// Página principal protegida
// async fn index_page(session: Session) -> impl Responder {
//     if session.get::<String>("auth_token").unwrap_or(None).is_some() {
//         HttpResponse::Ok()
//             .append_header(("Cache-Control", "max-age=31536000")) // 1 año
//             .append_header(("ETag", "custom-etag-value"))
//             .body("Página principal protegida")
//
//     } else {
//         HttpResponse::Found()
//             .append_header(("Location", "/login"))
//             .finish()
//     }
// }

async fn index_script() -> HttpResponse {
    file_cache::file_handler("./static/index/index_script.js")
}

// async fn login_page() -> HttpResponse {
//     //file_cache::file_handler(".static/login/login.html")
//     HttpResponse::Found()
//         .append_header(("login", ".static/login/login.html"))
//         .finish()
// }

async fn login_page() -> HttpResponse {
    let path = "./static/login/login.html"; // Ruta completa al archivo

    // Verificar si el archivo existe
    if !Path::new(path).is_file() {
        eprintln!("Archivo no encontrado: {}", path);
        return HttpResponse::NotFound().body("Archivo no encontrado");
    }

    println!("Sirviendo archivo desde /login: {}", path);
    file_cache::file_handler(path) // Sirve el archivo
}

async fn login_script() -> HttpResponse {
    file_cache::file_handler("./static/login/login_script.js")
}

async fn allcss_page() -> HttpResponse {
    file_cache::file_handler("./all.css")
}

// Página de error 404
async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("404 Página no encontrada")
}

async fn redirect_301() -> HttpResponse {
    HttpResponse::MovedPermanently()
        .append_header(("Location", "/nuevo-destino"))
        .finish()
}

async fn redirect_302() -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", "/temporal-destino"))
        .finish()
}

// fn file_handler() -> HttpResponse {
//     HttpResponse::Ok()
//         .append_header(("Cache-Control", "max-age=31536000")) // 1 año
//         .append_header(("ETag", "custom-etag-value"))
//         .body("Contenido del archivo estático")
// }

async fn items_handler(session: Session) -> impl Responder {
    if session.get::<String>("auth_token").unwrap_or(None).is_some() {
        HttpResponse::Ok().json(vec![
            "Item 1",
            "Item 2",
            "Item 3",
        ]) // Devuelve una lista de ítems como JSON
    } else {
        HttpResponse::Unauthorized().body("No autorizado")
    }
}

// Generar clave secreta para las sesiones
fn secret_key() -> Key {
    println!("Advertencia: Usando una clave secreta de prueba para las sesiones");
    Key::from(&[0; 64])
}

