use actix_web::{post, web, HttpResponse, Responder};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::database::User;
use crate::utils::auth::{create_token, AuthResponse};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

#[post("/login")]
pub async fn login(
    credentials: web::Json<LoginRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE username = $1",
        credentials.username
    )
    .fetch_optional(pool.get_ref())
    .await;

    match user {
        Ok(Some(user)) => {
            if verify(&credentials.password, &user.password_hash).unwrap_or(false) {
                let token = create_token(user.id, &user.username, user.is_admin)
                    .expect("Failed to create token");

                HttpResponse::Ok().json(AuthResponse {
                    token,
                    user_id: user.id,
                    username: user.username,
                    is_admin: user.is_admin,
                })
            } else {
                HttpResponse::Unauthorized().json("Invalid credentials")
            }
        }
        _ => HttpResponse::Unauthorized().json("Invalid credentials"),
    }
}

#[post("/register")]
pub async fn register(
    user_data: web::Json<RegisterRequest>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let password_hash = hash(&user_data.password, DEFAULT_COST).expect("Failed to hash password");

    match sqlx::query!(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id, username, is_admin
        "#,
        user_data.username,
        user_data.email,
        password_hash
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(user) => {
            let token = create_token(user.id, &user.username, user.is_admin)
                .expect("Failed to create token");

            HttpResponse::Created().json(AuthResponse {
                token,
                user_id: user.id,
                username: user.username,
                is_admin: user.is_admin,
            })
        }
        Err(e) => {
            if e.as_database_error()
                .and_then(|e| e.code())
                .map_or(false, |code| code == "23505")
            {
                HttpResponse::Conflict().json("Username or email already exists")
            } else {
                log::error!("Failed to register user: {}", e);
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}
