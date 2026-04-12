mod common;

use common::db::setup_test_db;
use common::fixtures::fixture_new_todo;
use domain::model::todo::{NewTodo, Todo, UpdateTodo, UpsertTodo};
use domain::model::Id;
use infra::repository::todo::status::TodoStatusRepository;
use infra::repository::todo::TodoRepository;

/// insert → get (id 조회)
#[tokio::test]
async fn insert_todo_stores_and_retrieves_by_id() {
    let pool = setup_test_db().await;
    let repo = TodoRepository::new(pool.clone());
    let mut tx = pool.begin().await.unwrap();

    let inserted = repo.insert_tx(&mut tx, fixture_new_todo()).await.unwrap();
    let found = repo.get_tx(&mut tx, &inserted.id).await.unwrap();

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
    let repo = TodoRepository::new(pool.clone());
    let mut tx = pool.begin().await.unwrap();

    let id: Id<Todo> = Id::gen();
    let found = repo.get_tx(&mut tx, &id).await.unwrap();

    assert!(found.is_none());
    tx.rollback().await.unwrap();
}

/// 여러 insert 후 find(None) → 전체 목록 포함 확인
#[tokio::test]
async fn find_todos_without_filter_returns_all_inserted() {
    let pool = setup_test_db().await;
    let repo = TodoRepository::new(pool.clone());
    let mut tx = pool.begin().await.unwrap();

    repo.insert_tx(&mut tx, fixture_new_todo()).await.unwrap();
    repo.insert_tx(
        &mut tx,
        NewTodo::new(
            Id::gen(),
            "Second Todo".to_string(),
            "Second Desc".to_string(),
        ),
    )
    .await
    .unwrap();

    let found = repo.find_tx(&mut tx, None).await.unwrap();
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
    let repo = TodoRepository::new(pool.clone());
    let mut tx = pool.begin().await.unwrap();

    let inserted = repo.insert_tx(&mut tx, fixture_new_todo()).await.unwrap();
    let update = UpdateTodo::new(inserted.id, Some("Updated Title".to_string()), None, None);
    let updated = repo.update_tx(&mut tx, update).await.unwrap();

    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.description, "Test Todo Description");
    tx.rollback().await.unwrap();
}

/// upsert: 새 레코드 insert
#[tokio::test]
async fn upsert_todo_inserts_new_record() {
    let pool = setup_test_db().await;
    let repo = TodoRepository::new(pool.clone());
    let status_repo = TodoStatusRepository::new(pool.clone());
    let mut tx = pool.begin().await.unwrap();

    let status = status_repo.get_by_code_tx(&mut tx, "new").await.unwrap();
    let upsert = UpsertTodo::new(
        Id::gen(),
        "Upserted Title".to_string(),
        "Upserted Desc".to_string(),
        status,
    );
    let result = repo.upsert_tx(&mut tx, upsert).await.unwrap();

    assert_eq!(result.title, "Upserted Title");
    assert_eq!(result.status.code, "new");
    tx.rollback().await.unwrap();
}

/// upsert: 같은 id로 재 upsert → 업데이트
#[tokio::test]
async fn upsert_todo_updates_existing_record() {
    let pool = setup_test_db().await;
    let repo = TodoRepository::new(pool.clone());
    let status_repo = TodoStatusRepository::new(pool.clone());
    let mut tx = pool.begin().await.unwrap();

    let status = status_repo.get_by_code_tx(&mut tx, "new").await.unwrap();
    let first_id: Id<Todo> = Id::gen();
    let first_id_value = first_id.value;

    let first = UpsertTodo::new(
        first_id,
        "Original Title".to_string(),
        "Original Desc".to_string(),
        status,
    );
    repo.upsert_tx(&mut tx, first).await.unwrap();

    let status2 = status_repo
        .get_by_code_tx(&mut tx, "working")
        .await
        .unwrap();
    let second = UpsertTodo::new(
        Id::new(first_id_value),
        "Updated Title".to_string(),
        "Updated Desc".to_string(),
        status2,
    );
    let result = repo.upsert_tx(&mut tx, second).await.unwrap();

    assert_eq!(result.id.value, first_id_value);
    assert_eq!(result.title, "Updated Title");
    assert_eq!(result.status.code, "working");
    tx.rollback().await.unwrap();
}

/// delete: 존재하는 todo 삭제 → 삭제된 todo 반환, 이후 조회 None
#[tokio::test]
async fn delete_todo_removes_and_returns_deleted_todo() {
    let pool = setup_test_db().await;
    let repo = TodoRepository::new(pool.clone());
    let mut tx = pool.begin().await.unwrap();

    let inserted = repo.insert_tx(&mut tx, fixture_new_todo()).await.unwrap();
    let id_value = inserted.id.value;

    let deleted = repo.delete_tx(&mut tx, &inserted.id).await.unwrap();
    assert!(deleted.is_some(), "delete should return the removed todo");
    assert_eq!(deleted.unwrap().id.value, id_value);

    let after = repo.get_tx(&mut tx, &inserted.id).await.unwrap();
    assert!(after.is_none(), "todo should not exist after deletion");
    tx.rollback().await.unwrap();
}

/// delete: 존재하지 않는 todo 삭제 → None 반환
#[tokio::test]
async fn delete_nonexistent_todo_returns_none() {
    let pool = setup_test_db().await;
    let repo = TodoRepository::new(pool.clone());
    let mut tx = pool.begin().await.unwrap();

    let id: Id<Todo> = Id::gen();
    let result = repo.delete_tx(&mut tx, &id).await.unwrap();

    assert!(result.is_none());
    tx.rollback().await.unwrap();
}
