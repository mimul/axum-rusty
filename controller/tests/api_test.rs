mod common;

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

fn unique_email() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("test_{n}@example.com")
}

async fn body_json(body: axum::body::Body) -> Value {
    let bytes = body
        .collect()
        .await
        .expect("body collection should not fail")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("response body should be valid JSON")
}

async fn create_user_and_login(app: &axum::Router, email: &str) -> String {
    let create_body = json!({
        "username": email,
        "password": "Test1234!",
        "fullname": "Test User"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/create")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(create_body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let json = body_json(resp.into_body()).await;
    assert_eq!(
        json["result"], true,
        "setup: create_user must succeed for {email}, got: {json}"
    );

    let login_body = json!({
        "username": email,
        "password": "Test1234!"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(login_body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let json = body_json(resp.into_body()).await;
    json["data"]["token"]
        .as_str()
        .expect("token not found in login response")
        .to_string()
}

// ─── health check ────────────────────────────────────────────────────────────

#[tokio::test]
async fn hc_returns_no_content() {
    let app = common::build_test_app().await;
    let req = Request::builder()
        .method(Method::GET)
        .uri("/v1/hc")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn hc_postgres_returns_no_content() {
    let app = common::build_test_app().await;
    let req = Request::builder()
        .method(Method::GET)
        .uri("/v1/hc/postgres")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ─── unknown route fallback ───────────────────────────────────────────────────
// AppError::Error → 200 OK (result: false)

#[tokio::test]
async fn unknown_route_returns_error_response() {
    let app = common::build_test_app().await;
    let req = Request::builder()
        .method(Method::GET)
        .uri("/v1/unknown-path")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}

// ─── unknown api version ─────────────────────────────────────────────────────
// AppError::UnknownApiVerRejection → 400

#[tokio::test]
async fn unknown_api_version_returns_bad_request() {
    let app = common::build_test_app().await;
    let req = Request::builder()
        .method(Method::GET)
        .uri("/v99/hc")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ─── user creation ───────────────────────────────────────────────────────────

#[tokio::test]
async fn create_user_with_valid_data_returns_ok() {
    let app = common::build_test_app().await;
    let email = unique_email();
    let body = json!({
        "username": email,
        "password": "Test1234!",
        "fullname": "Test User"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/create")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], true);
    assert!(json["data"]["userView"]["id"].is_string());
}

#[tokio::test]
async fn create_user_response_does_not_expose_password_hash() {
    // Arrange
    let app = common::build_test_app().await;
    let body = json!({
        "username": unique_email(),
        "password": "Test1234!",
        "fullname": "Security Test"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/create")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();

    // Act
    let resp = app.oneshot(req).await.unwrap();

    // Assert: 패스워드 해시 미노출 (§6.3)
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    let user_view = &json["data"]["userView"];
    assert!(
        user_view.get("password").is_none(),
        "password hash must not be exposed in API response, got: {user_view}"
    );
}

// AppError::Validation → 400
#[tokio::test]
async fn create_user_with_invalid_email_returns_bad_request() {
    let app = common::build_test_app().await;
    let body = json!({
        "username": "not-an-email",
        "password": "Test1234!",
        "fullname": "Test User"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/create")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}

#[tokio::test]
async fn create_user_with_weak_password_returns_bad_request() {
    let app = common::build_test_app().await;
    let body = json!({
        "username": unique_email(),
        "password": "weak",
        "fullname": "Test User"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/create")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}

#[tokio::test]
async fn create_user_with_missing_fullname_returns_bad_request() {
    let app = common::build_test_app().await;
    let body = json!({
        "username": unique_email(),
        "password": "Test1234!"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/create")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}

// ─── login ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn login_with_valid_credentials_returns_token() {
    let app = common::build_test_app().await;
    let email = unique_email();

    let create_body = json!({
        "username": email,
        "password": "Test1234!",
        "fullname": "Login Tester"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/create")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(create_body.to_string()))
        .unwrap();
    let _ = app.clone().oneshot(req).await.unwrap();

    let login_body = json!({
        "username": email,
        "password": "Test1234!"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(login_body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], true);
    assert!(json["data"]["token"].is_string());
}

// AppError::Error → 200 OK (result: false)
#[tokio::test]
async fn login_with_wrong_password_returns_error_result() {
    let app = common::build_test_app().await;
    let email = unique_email();

    let create_body = json!({
        "username": email,
        "password": "Test1234!",
        "fullname": "Login Tester"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/create")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(create_body.to_string()))
        .unwrap();
    let _ = app.clone().oneshot(req).await.unwrap();

    let login_body = json!({
        "username": email,
        "password": "Wrong1234!"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(login_body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}

// ─── auth middleware ──────────────────────────────────────────────────────────
// AppError::InvalidJwt → 400

#[tokio::test]
async fn protected_route_without_token_returns_bad_request() {
    let app = common::build_test_app().await;
    let req = Request::builder()
        .method(Method::GET)
        .uri("/v1/todo")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}

#[tokio::test]
async fn protected_route_with_invalid_token_returns_bad_request() {
    let app = common::build_test_app().await;
    let req = Request::builder()
        .method(Method::GET)
        .uri("/v1/todo")
        .header(header::AUTHORIZATION, "Bearer invalid.token.here")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}

// ─── todo CRUD ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn find_todo_with_valid_token_returns_ok() {
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/v1/todo?status=new")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], true);
}

#[tokio::test]
async fn create_todo_with_valid_token_returns_ok() {
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;

    let body = json!({
        "title": "Test Todo",
        "description": "Test Description"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/todo")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], true);
    assert!(json["data"]["todoView"]["id"].is_string());
}

// AppError::Validation → 400
#[tokio::test]
async fn create_todo_without_title_returns_bad_request() {
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;

    let body = json!({
        "description": "No title here"
    });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/todo")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}

#[tokio::test]
async fn get_todo_by_id_returns_ok() {
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;

    let body = json!({ "title": "Fetch Me", "description": "fetch description" });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/todo")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let json = body_json(resp.into_body()).await;
    let id = json["data"]["todoView"]["id"].as_str().unwrap().to_string();

    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("/v1/todo/{}", id))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], true);
    assert_eq!(json["data"]["todoView"]["id"], id);
}

#[tokio::test]
async fn update_todo_returns_ok() {
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;

    let body = json!({ "title": "Original Title", "description": "original description" });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/todo")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let json = body_json(resp.into_body()).await;
    let id = json["data"]["todoView"]["id"].as_str().unwrap().to_string();

    let update_body = json!({ "title": "Updated Title" });
    let req = Request::builder()
        .method(Method::PATCH)
        .uri(format!("/v1/todo/{}", id))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(update_body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], true);
}

#[tokio::test]
async fn upsert_todo_returns_ok() {
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;

    let body = json!({ "title": "Upsert Title", "description": "upsert description" });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/todo")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let json = body_json(resp.into_body()).await;
    let id = json["data"]["todoView"]["id"].as_str().unwrap().to_string();

    let upsert_body = json!({ "title": "Upserted Title", "description": "upserted description", "statusCode": "new" });
    let req = Request::builder()
        .method(Method::PUT)
        .uri(format!("/v1/todo/{}", id))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(upsert_body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], true);
}

#[tokio::test]
async fn delete_todo_returns_ok() {
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;

    let body = json!({ "title": "To Be Deleted", "description": "delete description" });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/todo")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let json = body_json(resp.into_body()).await;
    let id = json["data"]["todoView"]["id"].as_str().unwrap().to_string();

    let req = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/v1/todo/{}", id))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], true);
}

// ─── user read ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_user_by_id_with_valid_token_returns_ok() {
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;

    let login_body = json!({ "username": email, "password": "Test1234!" });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(login_body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let json = body_json(resp.into_body()).await;
    let user_id = json["data"]["userView"]["id"].as_str().unwrap().to_string();

    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("/v1/user/{}", user_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], true);
    assert_eq!(json["data"]["userView"]["id"], user_id);
}

#[tokio::test]
async fn get_user_by_username_with_valid_token_returns_ok() {
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;

    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("/v1/user?username={}", email))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], true);
}

// AppError::Error → 200 OK (result: false)
#[tokio::test]
async fn get_user_by_username_with_empty_username_returns_error_result() {
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/v1/user?username=")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}
