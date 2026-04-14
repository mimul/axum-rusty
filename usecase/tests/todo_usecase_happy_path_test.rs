//! TodoUseCase 정상 경로 통합 테스트
//!
//! 실행 방법:
//! ```
//! TEST_DATABASE_URL="postgresql://postgres:postgres@localhost:5432/todo_db" \
//!   cargo test -p usecase --test todo_usecase_happy_path_test -- --test-threads=1
//! ```

mod common;

use common::db::setup_test_db;
use common::module::build_usecase_test_module;
use shaku::HasComponent;
use std::sync::Arc;
use usecase::model::todo::{CreateTodo, SearchTodoCondition};
use usecase::usecase::todo::ITodoUseCase;

// ─── create_todo ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_todo_with_valid_input_stores_and_returns_todo() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn ITodoUseCase> = module.resolve();

    let source = CreateTodo::new("Buy milk".to_string(), "2 bottles".to_string());
    let result = uc.create_todo(source).await;

    let view = result.expect("create_todo must succeed with valid input");
    assert_eq!(view.title, "Buy milk");
    assert_eq!(view.description, "2 bottles");
    assert!(!view.id.is_empty(), "id must be assigned");
}

// ─── get_todo ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_todo_with_existing_id_returns_todo() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn ITodoUseCase> = module.resolve();

    let created = uc
        .create_todo(CreateTodo::new(
            "Read book".to_string(),
            "Rust book".to_string(),
        ))
        .await
        .expect("setup: create_todo must succeed");

    let result = uc.get_todo(created.id.clone()).await;
    let view = result
        .expect("get_todo must succeed")
        .expect("todo must be found");
    assert_eq!(view.id, created.id);
    assert_eq!(view.title, "Read book");
}

#[tokio::test]
async fn get_todo_with_nonexistent_id_returns_none() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn ITodoUseCase> = module.resolve();

    let fake_id = domain::model::Id::<domain::model::todo::Todo>::gen()
        .value
        .to_string();
    let result = uc.get_todo(fake_id).await;

    assert!(
        result.expect("get_todo must not error").is_none(),
        "nonexistent id must return None"
    );
}

// ─── find_todo ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn find_todo_without_filter_returns_created_todo() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn ITodoUseCase> = module.resolve();

    let created = uc
        .create_todo(CreateTodo::new(
            "__FIND_TEST_TODO__".to_string(),
            "find test".to_string(),
        ))
        .await
        .expect("setup: create_todo must succeed");

    let todos = uc
        .find_todo(SearchTodoCondition { status_code: None })
        .await
        .expect("find_todo must succeed");

    assert!(
        todos.iter().any(|t| t.id == created.id),
        "created todo must appear in find results"
    );
}

// ─── delete_todo ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_todo_removes_and_returns_deleted_todo() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn ITodoUseCase> = module.resolve();

    let created = uc
        .create_todo(CreateTodo::new("Delete me".to_string(), "temp".to_string()))
        .await
        .expect("setup: create_todo must succeed");

    let deleted = uc
        .delete_todo(created.id.clone())
        .await
        .expect("delete_todo must succeed")
        .expect("deleted todo must be returned");
    assert_eq!(deleted.id, created.id);

    let after = uc
        .get_todo(created.id)
        .await
        .expect("get_todo after delete must not error");
    assert!(after.is_none(), "todo must not exist after deletion");
}
