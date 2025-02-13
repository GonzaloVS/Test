use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use crate::models::user::User;

// Obtener todos los usuarios
async fn get_users(db_pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as::<_, User>("SELECT id, name, email FROM users")
        .fetch_all(db_pool.get_ref())
        .await;

    match result {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "No se pudieron obtener los datos"
        })),
    }
}

// Configurar rutas del módulo
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users") // `/v1/users`
            .route("", web::get().to(get_users)),
    );
}
