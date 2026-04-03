mod common;

use common::db::setup_test_db;
use domain::repository::todo::status::TodoStatusRepository;
use infra::repository::todo::status::TodoStatusRepositoryImpl;

/// 유효한 code → TodoStatus 반환
#[tokio::test]
async fn get_by_code_with_valid_code_returns_status() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = TodoStatusRepositoryImpl::new();

    let status = repo.get_by_code("new", &mut tx).await;

    assert!(status.is_ok(), "valid code 'new' should return a status");
    let status = status.unwrap();
    assert_eq!(status.code, "new");
    assert_eq!(status.name, "신규");
    tx.rollback().await.unwrap();
}

/// 존재하지 않는 code → 에러 반환
#[tokio::test]
async fn get_by_code_with_invalid_code_returns_error() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = TodoStatusRepositoryImpl::new();

    let result = repo.get_by_code("nonexistent_code_xyz", &mut tx).await;

    assert!(result.is_err(), "invalid code should return an error");
    tx.rollback().await.unwrap();
}
