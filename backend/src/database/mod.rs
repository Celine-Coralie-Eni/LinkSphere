use sqlx::postgres::PgPoolOptions;
use std::env;

pub mod schema;

pub async fn create_pool() -> sqlx::PgPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool")
}

pub async fn init_database(pool: &sqlx::PgPool) {
    sqlx::query(schema::CREATE_USERS_TABLE)
        .execute(pool)
        .await
        .expect("Failed to create users table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS links (
            id SERIAL PRIMARY KEY,
            title VARCHAR(255) NOT NULL,
            url TEXT NOT NULL,
            description TEXT,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create links table");
}
