use actix_web::dev::Payload;
use actix_web::{Error, FromRequest, HttpRequest};
use chrono::{Duration, Utc};
use futures::future::{ready, Ready};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32, // user id
    pub username: String,
    pub is_admin: bool,
    pub exp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: i32,
    pub username: String,
    pub is_admin: bool,
}

pub fn create_token(
    user_id: i32,
    username: &str,
    is_admin: bool,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::days(7))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id,
        username: username.to_owned(),
        is_admin,
        exp: expiration,
    };

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

pub struct AuthenticatedUser {
    pub user_id: i32,
    pub username: String,
    pub is_admin: bool,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization");

        if let Some(auth_header) = auth_header {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = auth_str[7..].trim();
                    if let Ok(claims) = validate_token(token) {
                        return ready(Ok(AuthenticatedUser {
                            user_id: claims.sub,
                            username: claims.username,
                            is_admin: claims.is_admin,
                        }));
                    }
                }
            }
        }

        ready(Err(actix_web::error::ErrorUnauthorized("Invalid token")))
    }
}
