use actix_web::{web, HttpResponse};
use chrono::Utc;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(
    form: web::Form<FormData>,
    db_pool: web::Data<sqlx::PgPool>,
) -> HttpResponse {
    // sqlx::query!(
    //     r#"
    //         INSERT into subscriptions (id, email, name, subscribed_at)
    //         VALUES ($1, $2, $3, $4)
    //     "#,
    //     Uuid::
    // );
    let result = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db_pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            eprintln!("Failed to execute query : {e}");
            // HttpResponse::InternalServerError().finish()
            HttpResponse::InternalServerError().body(format!("Database error: {}", e))
        }
    }
}
