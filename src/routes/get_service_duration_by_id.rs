use actix_web::web;
use sqlx::PgPool;
use std::error::Error;

pub async fn get_service_duration_by_id(
    db_pool: web::Data<PgPool>,
    service_id: i64,
) -> Result<i64, Box<dyn Error>> {
    let rec = sqlx::query!(
        "SELECT service_duration FROM services WHERE service_id = $1",
        service_id
    )
    .fetch_one(db_pool.get_ref())
    .await?;

    Ok(rec.service_duration.expect("err"))
}
