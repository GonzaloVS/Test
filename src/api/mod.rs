use actix_web::web;

mod items;
mod users;

use crate::utils::handlers::api_handler;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1") // Prefijo /v1 en todas las rutas
            .configure(items::config)
            .configure(users::config)
            .route("/{_:.*}", web::get().to(api_handler)), // Mantiene `api_handler` como fallback
    );
}
