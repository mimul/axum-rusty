//! TodoUseCase 정상 경로 통합 테스트
//!
//! 실행 방법:
//! ```
//! cargo test -p usecase --test todo_usecase_happy_path_test
//! ```
//!
//! Docker가 실행 중이면 PostgreSQL 컨테이너를 자동으로 기동한다.

mod common;

use common::db::setup_test_db;
use common::module::build_usecase_test_module;
use shaku::HasComponent;
use std::sync::Arc;
use usecase::model::todo::{CreateTodo, SearchTodoCondition, UpdateTodoView, UpsertTodoView};
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

// ─── update_todo ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn update_todo_with_title_change_returns_updated_view() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn ITodoUseCase> = module.resolve();

    let created = uc
        .create_todo(CreateTodo::new(
            "Original Title".to_string(),
            "Original Desc".to_string(),
        ))
        .await
        .expect("setup: create_todo must succeed");

    let update = UpdateTodoView::new(
        created.id.clone(),
        Some("Updated Title".to_string()),
        None,
        None,
    );
    let result = uc.update_todo(update).await;

    let view = result.expect("update_todo must succeed with valid input");
    assert_eq!(view.id, created.id);
    assert_eq!(view.title, "Updated Title");
    assert_eq!(
        view.description, "Original Desc",
        "description must be unchanged"
    );
}

#[tokio::test]
async fn update_todo_with_status_code_changes_status() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn ITodoUseCase> = module.resolve();

    let created = uc
        .create_todo(CreateTodo::new(
            "Status Test".to_string(),
            "desc".to_string(),
        ))
        .await
        .expect("setup: create_todo must succeed");

    let update = UpdateTodoView::new(created.id.clone(), None, None, Some("done".to_string()));
    let view = uc
        .update_todo(update)
        .await
        .expect("update_todo with valid status must succeed");
    assert_eq!(view.status.code, "done");
}

// ─── upsert_todo ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn upsert_todo_inserts_when_id_not_exists() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn ITodoUseCase> = module.resolve();

    let new_id = domain::model::Id::<domain::model::todo::Todo>::gen()
        .value
        .to_string();
    let upsert = UpsertTodoView::new(
        new_id.clone(),
        "Upserted Title".to_string(),
        "Upserted Desc".to_string(),
        "new".to_string(),
    );
    let view = uc
        .upsert_todo(upsert)
        .await
        .expect("upsert_todo must succeed on insert");
    assert_eq!(view.id, new_id);
    assert_eq!(view.title, "Upserted Title");
    assert_eq!(view.status.code, "new");
}

#[tokio::test]
async fn upsert_todo_updates_existing_todo() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn ITodoUseCase> = module.resolve();

    let created = uc
        .create_todo(CreateTodo::new(
            "Before Upsert".to_string(),
            "desc".to_string(),
        ))
        .await
        .expect("setup: create_todo must succeed");

    let upsert = UpsertTodoView::new(
        created.id.clone(),
        "After Upsert".to_string(),
        "new desc".to_string(),
        "working".to_string(),
    );
    let view = uc
        .upsert_todo(upsert)
        .await
        .expect("upsert_todo must succeed on update");
    assert_eq!(view.id, created.id);
    assert_eq!(view.title, "After Upsert");
    assert_eq!(view.status.code, "working");
}

// ─── find_todo (status filter) ────────────────────────────────────────────────

#[tokio::test]
async fn find_todo_with_status_code_filter_returns_only_matching_todos() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn ITodoUseCase> = module.resolve();

    // 기본 상태 "new"로 todo 생성
    let created = uc
        .create_todo(CreateTodo::new(
            "__FILTER_TEST_TODO__".to_string(),
            "filter test".to_string(),
        ))
        .await
        .expect("setup: create_todo must succeed");

    let todos = uc
        .find_todo(SearchTodoCondition {
            status_code: Some("new".to_string()),
        })
        .await
        .expect("find_todo with status filter must succeed");

    assert!(
        todos.iter().any(|t| t.id == created.id),
        "newly created todo must appear in 'new' status filter"
    );
    assert!(
        todos.iter().all(|t| t.status.code == "new"),
        "all results must have 'new' status code"
    );
}

// ─── 에러 케이스 ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_todo_with_invalid_ulid_format_returns_error() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn ITodoUseCase> = module.resolve();

    let result = uc.get_todo("not-a-valid-ulid".to_string()).await;
    assert!(
        result.is_err(),
        "invalid ULID format must return Err before DB call"
    );
}

#[tokio::test]
async fn delete_todo_with_nonexistent_valid_id_returns_none() {
    let pool = setup_test_db().await;
    let module = build_usecase_test_module(pool);
    let uc: Arc<dyn ITodoUseCase> = module.resolve();

    let nonexistent_id = domain::model::Id::<domain::model::todo::Todo>::gen()
        .value
        .to_string();
    let result = uc
        .delete_todo(nonexistent_id)
        .await
        .expect("delete of nonexistent id must not return Err");
    assert!(
        result.is_none(),
        "deleting nonexistent todo must return None"
    );
}
