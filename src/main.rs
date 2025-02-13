use actix_web::{web, App, HttpServer};
use std::env;

mod api;
mod db;
mod utils;
mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://gonzo:ZNTn3H2^)^88@57.128.171.242:5432/trancosdb".to_string()
    });

    let db_pool = db::connection::connect(&database_url).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .configure(api::config_routes) // Se cargan todas las rutas aquí
    })
        .bind("0.0.0.0:8000")?
        .run()
        .await
}
