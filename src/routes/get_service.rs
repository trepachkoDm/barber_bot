use crate::models::Services;
use actix_web::web;
use sqlx::PgPool;
use std::error::Error;

pub async fn get_service(db_pool: web::Data<PgPool>) -> Result<Vec<Services>, Box<dyn Error>> {
    let services = sqlx::query_as!(
        Services,
        "SELECT service_id, service_name, service_duration, service_price
        FROM services"
    )
    .fetch_all(db_pool.get_ref())
    .await?;

    Ok(services)
}
