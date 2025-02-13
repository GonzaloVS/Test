use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use crate::models::item::Item;

async fn get_items(db_pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as::<_, Item>("SELECT id, name, description FROM items")
        .fetch_all(db_pool.get_ref())
        .await;

    match result {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "No se pudieron obtener los datos"
        })),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/items") // `/v1/items`
            .route("", web::get().to(get_items)),
    );
}
