use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::env;

mod database;
mod routes;
mod utils;

use routes::{
    auth::{login, register},
    search::search,
};
use utils::rbac::{Role, RoleGuard};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let pool = database::create_pool().await;
    database::init_database(&pool).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(login)
            .service(register)
            .service(
                web::scope("/api")
                    .wrap(RoleGuard::new(Role::Admin))
                    .service(search),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
