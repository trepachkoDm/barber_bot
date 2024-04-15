use actix_web::web;
use sqlx::PgPool;
use std::error::Error;

pub async fn save_appointment(
    db_pool: web::Data<PgPool>,
    user_name: String,
    service_name: String,
    appointment_time: String,
) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
        "INSERT INTO appointments (user_name,
        service_name,
        appointment_time)
        VALUES ($1, $2, $3)",
        user_name,
        service_name,
        appointment_time,
    )
    .execute(db_pool.get_ref())
    .await
    .map_err(|e| Box::new(e) as Box<dyn Error>)?;

    Ok(())
}
