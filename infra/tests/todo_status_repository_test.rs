mod common;

use common::db::setup_test_db;
use infra::module::uow::PgTodoUnitOfWorkFactory;
use usecase::module::uow::TodoUnitOfWorkFactory;

/// 유효한 code → TodoStatus 반환
#[tokio::test]
async fn get_by_code_with_valid_code_returns_status() {
    let pool = setup_test_db().await;
    let factory = PgTodoUnitOfWorkFactory::new(pool);
    let mut uow = factory.begin().await.unwrap();

    let status = uow.todo_status_repo().get_by_code("new").await;

    assert!(status.is_ok(), "valid code 'new' should return a status");
    let status = status.unwrap();
    assert_eq!(status.code, "new");
    assert_eq!(status.name, "신규");
    uow.rollback().await.unwrap();
}

/// 존재하지 않는 code → 에러 반환
#[tokio::test]
async fn get_by_code_with_invalid_code_returns_error() {
    let pool = setup_test_db().await;
    let factory = PgTodoUnitOfWorkFactory::new(pool);
    let mut uow = factory.begin().await.unwrap();

    let result = uow.todo_status_repo().get_by_code("nonexistent_code_xyz").await;

    assert!(result.is_err(), "invalid code should return an error");
    uow.rollback().await.unwrap();
}
