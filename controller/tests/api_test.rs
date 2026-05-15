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
    let set_cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .expect("Set-Cookie header not found in login response");
    set_cookie
        .split(';')
        .next()
        .and_then(|part| part.strip_prefix("access_token="))
        .expect("access_token not found in Set-Cookie header")
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
    let set_cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .expect("Set-Cookie header must be present on login");
    assert!(
        set_cookie.contains("access_token="),
        "Set-Cookie must contain access_token, got: {set_cookie}"
    );
    assert!(set_cookie.contains("HttpOnly"), "cookie must be HttpOnly");
    assert!(set_cookie.contains("Secure"), "cookie must be Secure");
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
// AppError::InvalidJwt → 401

#[tokio::test]
async fn protected_route_without_token_returns_unauthorized() {
    let app = common::build_test_app().await;
    let req = Request::builder()
        .method(Method::GET)
        .uri("/v1/todo")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}

#[tokio::test]
async fn protected_route_with_invalid_token_returns_unauthorized() {
    let app = common::build_test_app().await;
    let req = Request::builder()
        .method(Method::GET)
        .uri("/v1/todo")
        .header(header::AUTHORIZATION, "Bearer invalid.token.here")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
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

// ─── find_todo: status 없이 호출 → 200 result:false ──────────────────────────

#[tokio::test]
async fn find_todo_without_status_returns_error_result() {
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;
    let req = Request::builder()
        .method(Method::GET)
        .uri("/v1/todo")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}

// ─── get_todo: 존재하지 않는 ID → 200 result:false ───────────────────────────

#[tokio::test]
async fn get_todo_with_nonexistent_id_returns_error_result() {
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;
    let fake_id = "00000000000000000000000001";
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("/v1/todo/{fake_id}"))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}

// ─── delete_todo: 존재하지 않는 ID → 200 result:false ────────────────────────

#[tokio::test]
async fn delete_todo_with_nonexistent_id_returns_error_result() {
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;
    let fake_id = "00000000000000000000000001";
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/v1/todo/{fake_id}"))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}

// ─── get_user: 다른 사용자 ID → 403 Forbidden ────────────────────────────────

#[tokio::test]
async fn get_user_with_different_user_id_returns_forbidden() {
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;
    let other_id = "00000000000000000000000001";
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("/v1/user/{other_id}"))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ─── create_user: 중복 username → 200 result:false ───────────────────────────

#[tokio::test]
async fn create_user_with_duplicate_username_returns_error_result() {
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
    let resp = app.clone().oneshot(req).await.unwrap();
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], true, "setup: first create must succeed");

    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/create")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}

// ─── Cookie 인증 → 200 result:true ───────────────────────────────────────────

#[tokio::test]
async fn protected_route_with_cookie_token_returns_ok() {
    let app = common::build_test_app().await;
    let email = unique_email();

    let body = json!({ "username": email, "password": "Test1234!", "fullname": "Test User" });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/create")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], true, "setup: create_user must succeed");
    let user_id = json["data"]["userView"]["id"].as_str().unwrap().to_string();

    let login = json!({ "username": email, "password": "Test1234!" });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(login.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let set_cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .expect("Set-Cookie header must be present on login");
    let token = set_cookie
        .split(';')
        .next()
        .and_then(|part| part.strip_prefix("access_token="))
        .expect("access_token not found in Set-Cookie")
        .to_string();

    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("/v1/user/{user_id}"))
        .header(header::COOKIE, format!("access_token={token}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], true, "cookie auth must succeed: {json}");
}

// ─── JsonRejection: Content-Type 없이 POST → 400 ────────────────────────────

#[tokio::test]
async fn create_todo_without_content_type_returns_bad_request() {
    // Arrange
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;

    // Act: Content-Type 헤더 없이 요청 → AppError::JsonRejection → 400
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/todo")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from(r#"{"title":"Test","description":"desc"}"#))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    // Assert
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}

// ─── update_todo: 빈 title → validation error → 200 result:false ─────────────

#[tokio::test]
async fn update_todo_with_empty_title_returns_error_result() {
    // Arrange
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;

    let body = json!({ "title": "Original", "description": "desc" });
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/todo")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let json = body_json(resp.into_body()).await;
    let id = json["data"]["todoView"]["id"].as_str().unwrap().to_string();

    // Act: 빈 title → JsonUpdateTodoContents.validate() → Err → AppError::Error
    let update_body = json!({ "title": "" });
    let req = Request::builder()
        .method(Method::PATCH)
        .uri(format!("/v1/todo/{id}"))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(update_body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    // Assert: AppError::Error → 200, result: false, message에 "title" 포함
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
    assert!(
        json["message"]
            .as_str()
            .map(|m| m.contains("title"))
            .unwrap_or(false),
        "error message should mention 'title', got: {json}"
    );
}

// ─── get_user_by_username: 다른 username → 403 ───────────────────────────────

#[tokio::test]
async fn get_user_by_username_with_different_username_returns_forbidden() {
    // Arrange
    let app = common::build_test_app().await;
    let email = unique_email();
    let token = create_user_and_login(&app, &email).await;
    let other_email = unique_email();

    // Act: current_user.username != query.username → AppError::Forbidden → 403
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("/v1/user?username={other_email}"))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    // Assert
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ─── auth: 유효 JWT이지만 DB에 유저 없음 → 401 ───────────────────────────────

#[tokio::test]
async fn protected_route_returns_unauthorized_when_user_not_in_db() {
    // Arrange: 유효하게 서명되었지만 DB에 존재하지 않는 유저 ID의 JWT
    let app = common::build_test_app().await;
    let ghost_token = common::create_jwt_for_nonexistent_user();

    // Act
    let req = Request::builder()
        .method(Method::GET)
        .uri("/v1/todo")
        .header(header::AUTHORIZATION, format!("Bearer {ghost_token}"))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();

    // Assert: auth_resolver.rs L58 → .ok_or_else(|| InvalidJwt("user not found")) → 401
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["result"], false);
}
