use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::utils::auth::AuthenticatedUser;
use crate::utils::rbac::{Role, RoleGuard};

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    id: i32,
    title: String,
    url: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
}

#[get("/search")]
pub async fn search(
    query: web::Query<SearchQuery>,
    pool: web::Data<PgPool>,
    _user: AuthenticatedUser,
) -> impl Responder {
    let search_term = format!("%{}%", query.q);

    match sqlx::query_as!(
        Link,
        r#"
        SELECT id, title, url, description, created_at
        FROM links
        WHERE title ILIKE $1 OR description ILIKE $1
        ORDER BY created_at DESC
        "#,
        search_term
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(links) => HttpResponse::Ok().json(links),
        Err(e) => {
            log::error!("Failed to execute search query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
