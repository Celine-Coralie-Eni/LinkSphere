use actix_web::{test, web, App};
use backend::{
    database,
    routes::{
        auth::{login, register},
        search::search,
    },
};
use serde_json::json;

#[actix_web::test]
async fn test_auth_and_search_flow() {
    // Initialize test database
    let pool = database::create_pool().await;
    database::init_database(&pool).await;

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(register)
            .service(login)
            .service(search),
    )
    .await;

    // Test 1: Register a new user
    let register_data = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    let register_req = test::TestRequest::post()
        .uri("/register")
        .set_json(register_data)
        .to_request();

    let register_resp = test::call_service(&app, register_req).await;
    assert!(register_resp.status().is_success(), "Registration failed");

    let register_body: serde_json::Value = test::read_body_json(register_resp).await;
    assert!(
        register_body["token"].is_string(),
        "No token in registration response"
    );
    assert!(
        !register_body["is_admin"].as_bool().unwrap(),
        "New user should not be admin"
    );

    // Test 2: Login with the registered user
    let login_data = json!({
        "username": "testuser",
        "password": "password123"
    });

    let login_req = test::TestRequest::post()
        .uri("/login")
        .set_json(login_data)
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    assert!(login_resp.status().is_success(), "Login failed");

    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let token = login_body["token"].as_str().unwrap();

    // Test 3: Try to access search without token (should fail)
    let search_req = test::TestRequest::get()
        .uri("/api/search?q=test")
        .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert!(
        search_resp.status().is_client_error(),
        "Unauthorized access should fail"
    );

    // Test 4: Try to access search with token (should succeed)
    let search_req = test::TestRequest::get()
        .uri("/api/search?q=test")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert!(
        search_resp.status().is_success(),
        "Authorized search failed"
    );

    let search_body: serde_json::Value = test::read_body_json(search_resp).await;
    assert!(search_body.is_array(), "Search response should be an array");
}

#[actix_web::test]
async fn test_admin_privileges() {
    // Initialize test database
    let pool = database::create_pool().await;
    database::init_database(&pool).await;

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(register)
            .service(login)
            .service(search),
    )
    .await;

    // Create an admin user directly in the database
    sqlx::query!(
        r#"
        INSERT INTO users (username, email, password_hash, is_admin)
        VALUES ($1, $2, $3, $4)
        "#,
        "adminuser",
        "admin@example.com",
        bcrypt::hash("adminpass", bcrypt::DEFAULT_COST).unwrap(),
        true
    )
    .execute(&pool)
    .await
    .unwrap();

    // Login as admin
    let login_data = json!({
        "username": "adminuser",
        "password": "adminpass"
    });

    let login_req = test::TestRequest::post()
        .uri("/login")
        .set_json(login_data)
        .to_request();

    let login_resp = test::call_service(&app, login_req).await;
    assert!(login_resp.status().is_success(), "Admin login failed");

    let login_body: serde_json::Value = test::read_body_json(login_resp).await;
    let admin_token = login_body["token"].as_str().unwrap();
    assert!(
        login_body["is_admin"].as_bool().unwrap(),
        "User should be admin"
    );

    // Test admin access to search
    let search_req = test::TestRequest::get()
        .uri("/api/search?q=test")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .to_request();

    let search_resp = test::call_service(&app, search_req).await;
    assert!(
        search_resp.status().is_success(),
        "Admin search access failed"
    );
}
