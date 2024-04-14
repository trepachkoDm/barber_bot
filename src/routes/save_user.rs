use actix_web::error::ErrorInternalServerError;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use std::error::Error;

pub async fn save_user(
    db_pool: web::Data<PgPool>,
    user_name: String,
    telegram_id: i64,
) -> Result<HttpResponse, Box<dyn Error>> {
    let query_result = sqlx::query!(
        "INSERT INTO users (user_name, telegram_id)
        VALUES ($1, $2)
        RETURNING user_id
        ",
        user_name,
        telegram_id
    )
    .fetch_one(db_pool.get_ref())
    .await
    .map_err(|e| {
        eprintln!("Failed to execute query: {:?}", e);
        ErrorInternalServerError(e)
    })?;
    Ok(HttpResponse::Ok().json(query_result.user_id))
}
