mod common;

use common::db::setup_test_db;
use common::fixtures::fixture_new_todo;
use domain::model::todo::{NewTodo, Todo, UpdateTodo, UpsertTodo};
use domain::model::Id;
use domain::repository::todo::status::TodoStatusRepository;
use domain::repository::todo::TodoRepository;
use infra::repository::todo::status::TodoStatusRepositoryImpl;
use infra::repository::todo::TodoRepositoryImpl;

/// insert → get (id 조회)
#[tokio::test]
async fn insert_todo_stores_and_retrieves_by_id() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = TodoRepositoryImpl::new();

    let inserted = repo.insert(fixture_new_todo(), &mut tx).await.unwrap();
    let found = repo.get(&inserted.id, &mut tx).await.unwrap();

    assert!(found.is_some(), "inserted todo should be retrievable by id");
    let found = found.unwrap();
    assert_eq!(found.id.value, inserted.id.value);
    assert_eq!(found.title, "Test Todo Title");
    assert_eq!(found.description, "Test Todo Description");
    tx.rollback().await.unwrap();
}

/// 존재하지 않는 id → None 반환
#[tokio::test]
async fn get_todo_with_nonexistent_id_returns_none() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = TodoRepositoryImpl::new();

    let id: Id<Todo> = Id::gen();
    let found = repo.get(&id, &mut tx).await.unwrap();

    assert!(found.is_none());
    tx.rollback().await.unwrap();
}

/// 여러 insert 후 find(None) → 전체 목록 포함 확인
#[tokio::test]
async fn find_todos_without_filter_returns_all_inserted() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = TodoRepositoryImpl::new();

    repo.insert(fixture_new_todo(), &mut tx).await.unwrap();
    repo.insert(
        NewTodo::new(Id::gen(), "Second Todo".to_string(), "Second Desc".to_string()),
        &mut tx,
    )
    .await
    .unwrap();

    let found = repo.find(None, &mut tx).await.unwrap();
    assert!(
        found.len() >= 2,
        "find(None) should return at least the 2 inserted todos"
    );
    tx.rollback().await.unwrap();
}

/// update: title 변경
#[tokio::test]
async fn update_todo_title_updates_correctly() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = TodoRepositoryImpl::new();

    let inserted = repo.insert(fixture_new_todo(), &mut tx).await.unwrap();
    let update = UpdateTodo::new(inserted.id, Some("Updated Title".to_string()), None, None);
    let updated = repo.update(update, &mut tx).await.unwrap();

    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.description, "Test Todo Description");
    tx.rollback().await.unwrap();
}

/// upsert: 새 레코드 insert
#[tokio::test]
async fn upsert_todo_inserts_new_record() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();
    let todo_repo = TodoRepositoryImpl::new();
    let status_repo = TodoStatusRepositoryImpl::new();

    let status = status_repo.get_by_code("new", &mut tx).await.unwrap();
    let upsert = UpsertTodo::new(
        Id::gen(),
        "Upserted Title".to_string(),
        "Upserted Desc".to_string(),
        status,
    );
    let result = todo_repo.upsert(upsert, &mut tx).await.unwrap();

    assert_eq!(result.title, "Upserted Title");
    assert_eq!(result.status.code, "new");
    tx.rollback().await.unwrap();
}

/// upsert: 같은 id로 재 upsert → 업데이트
#[tokio::test]
async fn upsert_todo_updates_existing_record() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();
    let todo_repo = TodoRepositoryImpl::new();
    let status_repo = TodoStatusRepositoryImpl::new();

    let status = status_repo.get_by_code("new", &mut tx).await.unwrap();
    let first_id: Id<Todo> = Id::gen();
    let first_id_value = first_id.value;

    let first = UpsertTodo::new(
        first_id,
        "Original Title".to_string(),
        "Original Desc".to_string(),
        status,
    );
    let _ = todo_repo.upsert(first, &mut tx).await.unwrap();

    let status2 = status_repo.get_by_code("working", &mut tx).await.unwrap();
    let second = UpsertTodo::new(
        Id::new(first_id_value),
        "Updated Title".to_string(),
        "Updated Desc".to_string(),
        status2,
    );
    let result = todo_repo.upsert(second, &mut tx).await.unwrap();

    assert_eq!(result.id.value, first_id_value);
    assert_eq!(result.title, "Updated Title");
    assert_eq!(result.status.code, "working");
    tx.rollback().await.unwrap();
}

/// delete: 존재하는 todo 삭제 → 삭제된 todo 반환, 이후 조회 None
#[tokio::test]
async fn delete_todo_removes_and_returns_deleted_todo() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = TodoRepositoryImpl::new();

    let inserted = repo.insert(fixture_new_todo(), &mut tx).await.unwrap();
    let id_value = inserted.id.value;

    let deleted = repo.delete(&inserted.id, &mut tx).await.unwrap();
    assert!(deleted.is_some(), "delete should return the removed todo");
    assert_eq!(deleted.unwrap().id.value, id_value);

    // 삭제 후 조회 → None
    let after = repo.get(&inserted.id, &mut tx).await.unwrap();
    assert!(after.is_none(), "todo should not exist after deletion");
    tx.rollback().await.unwrap();
}

/// delete: 존재하지 않는 todo 삭제 → None 반환
#[tokio::test]
async fn delete_nonexistent_todo_returns_none() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = TodoRepositoryImpl::new();

    let id: Id<Todo> = Id::gen();
    let result = repo.delete(&id, &mut tx).await.unwrap();

    assert!(result.is_none());
    tx.rollback().await.unwrap();
}
