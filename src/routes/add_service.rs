use actix_web::{web, HttpResponse, Responder};

use crate::models::Services;
use sqlx::PgPool;

pub async fn add_service(pool: web::Data<PgPool>, path: web::Path<Services>) -> impl Responder {
    let service_data = path.into_inner();
    match sqlx::query!(
        "INSERT INTO services (service_id, service_name, service_duration) VALUES ($1, $2, $3)",
        service_data.service_id,
        service_data.service_name,
        service_data.service_duration
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().body("Service added successfully"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to add service: {}", e)),
    }
}
