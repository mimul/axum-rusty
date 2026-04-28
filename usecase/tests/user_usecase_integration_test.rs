//! UserUseCase 통합 테스트
//!
//! 실행 방법:
//! ```
//! cargo test -p usecase --test user_usecase_integration_test
//! ```
//!
//! Docker가 실행 중이면 PostgreSQL 컨테이너를 자동으로 기동한다.

mod common;

use common::db::setup_test_db;
use common::module::build_usecase_test_module;
use shaku::HasComponent;
use std::sync::Arc;
use usecase::model::user::{CreateUser, LoginUser, SearchUserCondition};
use usecase::usecase::user::IUserUseCase;

fn unique_username(prefix: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}_{n}")
}

// ─── create_user ────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_user_with_valid_input_returns_user_view() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn IUserUseCase> = module.resolve();

    let username = unique_username("create_ok");
    let source = CreateUser::new(
        username.clone(),
        "password123".to_string(),
        "Test User".to_string(),
    );
    let result = uc.create_user(source).await;

    let view = result.expect("create_user must succeed with valid input");
    assert_eq!(view.username, username);
    assert_eq!(view.fullname, "Test User");
    assert!(!view.id.is_empty(), "id must be set");
}

#[tokio::test]
async fn create_user_with_duplicate_username_returns_error() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn IUserUseCase> = module.resolve();

    let username = unique_username("dup_user");
    let first = CreateUser::new(username.clone(), "pw1".to_string(), "First".to_string());
    uc.create_user(first)
        .await
        .expect("first create must succeed");

    let second = CreateUser::new(username.clone(), "pw2".to_string(), "Second".to_string());
    let result = uc.create_user(second).await;
    assert!(result.is_err(), "duplicate username must return Err");

    let msg = result.unwrap_err().to_string();
    assert_eq!(
        msg, "이미 사용 중인 사용자명입니다",
        "error must not expose username: got {msg}"
    );
}

// ─── login_user ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn login_user_with_valid_credentials_returns_user_view() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn IUserUseCase> = module.resolve();

    let username = unique_username("login_ok");
    let source = CreateUser::new(
        username.clone(),
        "correct_pw".to_string(),
        "Login User".to_string(),
    );
    uc.create_user(source)
        .await
        .expect("setup: create_user must succeed");

    let login = LoginUser::new(username.clone(), "correct_pw".to_string());
    let result = uc.login_user(login).await;

    let view = result.expect("login must succeed with correct credentials");
    assert_eq!(view.username, username);
}

#[tokio::test]
async fn login_user_with_wrong_password_returns_uniform_error() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn IUserUseCase> = module.resolve();

    let username = unique_username("login_bad_pw");
    let source = CreateUser::new(
        username.clone(),
        "correct_pw".to_string(),
        "Test".to_string(),
    );
    uc.create_user(source)
        .await
        .expect("setup: create_user must succeed");

    let login = LoginUser::new(username, "wrong_password".to_string());
    let result = uc.login_user(login).await;
    assert!(result.is_err(), "wrong password must return Err");

    let msg = result.unwrap_err().to_string();
    assert_eq!(
        msg, "잘못된 사용자명 또는 비밀번호입니다",
        "error message must be uniform to prevent user enumeration: got {msg}"
    );
}

#[tokio::test]
async fn login_user_with_unknown_username_returns_uniform_error() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn IUserUseCase> = module.resolve();

    let login = LoginUser::new("nonexistent_user_xyz".to_string(), "any_pw".to_string());
    let result = uc.login_user(login).await;
    assert!(result.is_err(), "unknown username must return Err");

    let msg = result.unwrap_err().to_string();
    assert_eq!(
        msg, "잘못된 사용자명 또는 비밀번호입니다",
        "error message must match wrong-password message to prevent enumeration: got {msg}"
    );
}

// ─── get_user ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_user_with_existing_id_returns_user_view() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn IUserUseCase> = module.resolve();

    let username = unique_username("get_user_ok");
    let created = uc
        .create_user(CreateUser::new(
            username.clone(),
            "pw".to_string(),
            "Get User".to_string(),
        ))
        .await
        .expect("setup: create_user must succeed");

    let result = uc.get_user(created.id.clone()).await;
    let view = result
        .expect("get_user must succeed")
        .expect("user must be found");
    assert_eq!(view.username, username);
}

#[tokio::test]
async fn get_user_with_nonexistent_id_returns_none() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn IUserUseCase> = module.resolve();

    let fake_id = domain::model::Id::<domain::model::user::User>::gen()
        .value
        .to_string();
    let result = uc.get_user(fake_id).await;

    assert!(
        result.expect("get_user must not error").is_none(),
        "nonexistent id must return None"
    );
}

// ─── get_user_by_username ────────────────────────────────────────────────────

#[tokio::test]
async fn get_user_by_username_with_existing_username_returns_user() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn IUserUseCase> = module.resolve();

    let username = unique_username("get_by_uname_ok");
    uc.create_user(CreateUser::new(
        username.clone(),
        "pw".to_string(),
        "Name".to_string(),
    ))
    .await
    .expect("setup: create_user must succeed");

    let cond = SearchUserCondition {
        username: Some(username.clone()),
    };
    let result = uc.get_user_by_username(cond).await;
    let view = result
        .expect("get_user_by_username must succeed")
        .expect("user must be found");
    assert_eq!(view.username, username);
}

#[tokio::test]
async fn get_user_by_username_with_none_returns_error() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn IUserUseCase> = module.resolve();

    let cond = SearchUserCondition { username: None };
    let result = uc.get_user_by_username(cond).await;
    assert!(result.is_err(), "None username must return Err");
}

// ─── 에러 케이스 ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_user_with_invalid_ulid_format_returns_error() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn IUserUseCase> = module.resolve();

    let result = uc.get_user("not-a-valid-ulid".to_string()).await;
    assert!(
        result.is_err(),
        "invalid ULID format must return Err before DB call"
    );
}

#[tokio::test]
async fn get_user_by_username_with_empty_string_returns_none() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn IUserUseCase> = module.resolve();

    let cond = SearchUserCondition {
        username: Some("__nonexistent_user_xyz_999__".to_string()),
    };
    let result = uc
        .get_user_by_username(cond)
        .await
        .expect("get_user_by_username with nonexistent name must not error");
    assert!(result.is_none(), "nonexistent username must return None");
}
