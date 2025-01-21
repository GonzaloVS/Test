mod css_utils;
mod error_utils;
mod file_cache;
mod file_utils;
mod metrics;

use crate::metrics::{export_metrics, Metrics};
use actix_cors::Cors;
use actix_session::storage::CookieSessionStore;
use actix_session::{Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{middleware, web, web::Data, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, };
use governor::clock::{ QuantaClock};
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use std::io;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use actix_web::body::{MessageBody};
use actix_web::middleware::Next;
use governor::middleware::NoOpMiddleware;


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

    // Iniciar el monitoreo de cambios
    tokio::spawn(async move {
        if let Err(e) = css_utils::monitor_changes(css_dir, output_file).await {
            eprintln!("Error en el monitoreo de cambios: {}", e);
        }
    });

    //env_logger::init(); // Inicializa logs

    // Configuración de direcciones y puertos
    let http_addr = "127.0.0.1:80";
    //let https_addr = "127.0.0.1:443";

    // tokio::join!(
    //     start_http_server(http_addr),
    //     //start_https_server(https_addr)
    // );

    // Iniciar el servidor HTTP
    HttpServer::new(move || {
        let metrics = metrics.clone();
        App::new()

            .route("/", web::get().to(index_page))
            // .route("/index.js", web::get().to(index_script))
            // .route("/login", web::get().to(login_page))
            // .route("/login.js", web::get().to(login_script))
            // .route("/all.css", web::get().to(allcss_page))
            // .route("/antigua-url", web::get().to(redirect_301))
            // .route("/temporal-url", web::get().to(redirect_302))
            // .route("/items", web::get().to(items_handler))
            // .route("/static/{filename:.*}", web::get().to(static_files))

            // .wrap(
            //     SessionMiddleware::builder(CookieSessionStore::default(), secret_key())
            //         .cookie_same_site(actix_web::cookie::SameSite::Strict)
            //         .build(),
            // )
            // .wrap(middleware::from_fn(rate_limit_middleware))
            // //.wrap_fn(|req, srv| rate_limit_middleware(req, srv))
            // .wrap(
            //     Cors::default()
            //         .allowed_origin("https://example.com")
            //         .allowed_methods(vec!["GET", "POST"])
            //         .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE])
            //         .max_age(3600),
            // )
            // .wrap(middleware::DefaultHeaders::new().add(("X-Example-Header", "Value")))
            // .wrap(middleware::Compress::default())
            // .app_data(web::Data::new(metrics.clone()))
            // .route(
            //     "/metrics",
            //     web::get().to(move || {
            //         let registry = metrics.registry.clone(); // Usa el registry compartido
            //         async move { export_metrics(registry).await }
            //     }),
            // )

            // .default_service(web::route().to(not_found))
            // .app_data(web::JsonConfig::default().error_handler(|err, _req| {
            //     actix_web::error::InternalError::from_response(
            //         err,
            //         error_utils::handle_400_error().into(),
            //     )
            //     .into()
            // }))
    })
    .bind(http_addr)?
    .workers(8)
    .max_connections(50_000)
    .max_connection_rate(1_000)
    .client_request_timeout(Duration::from_secs(30))
    .client_disconnect_timeout(Duration::from_secs(5))
    .run()
    .await?;

    // Iniciar servidor HTTPS (comentado, activar cuando sea necesario)
    /*
    HttpServer::new( move || {
         let metrics = metrics.clone();
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
     })
     .bind_openssl(https_addr, load_ssl_keys()?)?
     .workers(8)
     .max_connections(50_000)
     .max_connection_rate(1_000)
     .client_request_timeout(Duration::from_secs(30))
     .client_disconnect_timeout(Duration::from_secs(5))
     .run()
     .await?;
     */

    Ok(())
}

async fn rate_limit_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {

    let max_global_connections = Quota::per_minute(NonZeroU32::new(100).unwrap());
    Arc::new(RateLimiter::direct(max_global_connections));

    let global_limiter = req
        .app_data::<Data<Arc<RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware>>>>()
        .unwrap();

    //Verificar el limitador global
    if global_limiter.check().is_err() { return Err(actix_web::error::ErrorTooManyRequests("Global rate limit exceeded")); }

    // invoke the wrapped middleware or service
    let res = next.call(req).await?;

    // post-processing
    Ok(res)




    // Obtener la dirección IP del cliente
    // let client_ip = req
    //     .connection_info()
    //     .realip_remote_addr()
    //     .unwrap_or("unknown")
    //     .to_string();
    //

    // invoke the wrapped middleware or service
    // let res = next.call(req).await?;
    //
    // // post-processing
    //
    // Ok(res)
    //
    //

//
    //
    // let max_client_connections = Quota::per_minute(NonZeroU32::new(100).unwrap());
    // Arc::new(RateLimiter::keyed(max_client_connections));
    //
    // println!("Middleware: Procesando solicitud para {}", req.path());
    //
    // // Obtener el limitador global
    //
    //
    // // Obtener el limitador por cliente
    // let client_limiter = req
    //     .app_data::<Data<Arc<RateLimiter<String, DefaultKeyedStateStore<String>, QuantaClock>>>>()
    //     .unwrap();
    //
    // // Obtener la dirección IP del cliente
    // let client_ip = req
    //     .connection_info()
    //     .realip_remote_addr()
    //     .unwrap_or("unknown")
    //     .to_string();
    //
    //
    //
    // // Verificar el limitador por cliente
    // if client_limiter.check_key(&client_ip).is_err() {
    //     return Ok(req.into_response(
    //         HttpResponse::TooManyRequests()
    //             .body("Client rate limit exceeded")
    //             .map_into_boxed_body(),
    //     ));
    // }
    //
    // // Continuar con el siguiente middleware o controlador
    // srv.call(req).await
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

    println!(
        "RUTA GENÉRICA: Solicitado: {}, Mapeado a: {}",
        filename, path
    );

    if Path::new(&path).exists() {
        file_cache::file_handler(&path)
    } else {
        HttpResponse::NotFound().body("Archivo no encontrado")
    }
}


async fn index_page(metrics: web::Data<Metrics>, session: Session) -> impl Responder {
    metrics.http_requests_total.inc(); // Incrementar contador de solicitudes
    let timer = metrics.request_duration.start_timer(); // Iniciar temporizador

    let response = if session
        .get::<String>("auth_token")
        .unwrap_or(None)
        .is_some()
    {
        file_cache::file_handler("./static/index/index.html")
    } else {
        HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish()
    };

    timer.observe_duration(); // Registrar duración de la solicitud
    response
}

async fn index_script() -> HttpResponse {
    file_cache::file_handler("./static/index/index_script.js")
}

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
    if session
        .get::<String>("auth_token")
        .unwrap_or(None)
        .is_some()
    {
        HttpResponse::Ok().json(vec!["Item 1", "Item 2", "Item 3"]) // Devuelve una lista de ítems como JSON
    } else {
        HttpResponse::Unauthorized().body("No autorizado")
    }
}

// Generar clave secreta para las sesiones
fn secret_key() -> Key {
    println!("Advertencia: Usando una clave secreta de prueba para las sesiones");
    Key::from(&[0; 64])
}
