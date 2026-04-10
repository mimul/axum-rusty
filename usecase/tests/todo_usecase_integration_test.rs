mod common;

use common::db::setup_test_db;
use domain::model::todo::NewTodo;
use domain::model::Id;
use domain::repository::todo::TodoRepository;
use infra::module::repo_module::RepositoriesModule;
use infra::persistence::postgres::Db;
use infra::repository::todo::TodoRepositoryImpl;
use std::sync::Arc;
use usecase::model::todo::UpdateTodoView;
use usecase::usecase::todo::TodoUseCase;

/// update_todo에 존재하지 않는 status_code를 넘기면
/// get_by_code가 Err를 반환하고 트랜잭션이 롤백되어
/// DB의 todo 데이터가 변경되지 않음을 검증한다.
#[tokio::test]
async fn update_todo_with_invalid_status_rolls_back_transaction() {
    let pool = setup_test_db().await;

    // Setup: 테스트용 todo를 커밋하여 DB에 영구 저장
    let todo_repo = TodoRepositoryImpl::new();
    let mut setup_tx = pool.begin().await.unwrap();
    let new_todo = NewTodo::new(
        Id::gen(),
        "Original Title".to_string(),
        "Original Description".to_string(),
    );
    let inserted = todo_repo.insert(new_todo, &mut setup_tx).await.unwrap();
    let inserted_id = inserted.id.value.to_string();
    setup_tx.commit().await.unwrap();

    // Act: 존재하지 않는 status_code로 update_todo 호출
    // → get_by_code 실패 → ? 전파 → tx drop → 자동 롤백
    let db = Db(Arc::new(pool.clone()));
    let repos = Arc::new(RepositoriesModule::new());
    let usecase = TodoUseCase::new(db, repos);

    let update_view = UpdateTodoView::new(
        inserted_id,
        Some("Should Not Be Saved".to_string()),
        None,
        Some("INVALID_STATUS_THAT_DOES_NOT_EXIST".to_string()),
    );
    let result = usecase.update_todo(update_view).await;

    // Assert: 에러 반환 확인
    assert!(result.is_err(), "invalid status_code must return Err");

    // Assert: DB에 변경 없음 (롤백 검증)
    let mut verify_tx = pool.begin().await.unwrap();
    let found = todo_repo
        .get(&inserted.id, &mut verify_tx)
        .await
        .unwrap();
    verify_tx.rollback().await.unwrap();

    let found = found.expect("todo must still exist after rollback");
    assert_eq!(
        found.title, "Original Title",
        "title must be unchanged after transaction rollback"
    );

    // Cleanup: 테스트 데이터 삭제
    let mut cleanup_tx = pool.begin().await.unwrap();
    todo_repo.delete(&inserted.id, &mut cleanup_tx).await.unwrap();
    cleanup_tx.commit().await.unwrap();
}

/// update_todo에 존재하지 않는 todo ID를 넘기면
/// todo_repository().update() 내 fetch_one이 RowNotFound를 반환하고
/// 트랜잭션이 롤백되어 DB 상태가 변경되지 않음을 검증한다.
#[tokio::test]
async fn update_todo_with_nonexistent_id_rolls_back_transaction() {
    let pool = setup_test_db().await;
    let db = Db(Arc::new(pool.clone()));
    let repos = Arc::new(RepositoriesModule::new());
    let usecase = TodoUseCase::new(db, repos);

    // 존재하지 않는 ID로 update 시도 (status_code는 None → update()까지 진행)
    let nonexistent_id = Id::<domain::model::todo::Todo>::gen().value.to_string();
    let update_view = UpdateTodoView::new(
        nonexistent_id,
        Some("Ghost Title".to_string()),
        None,
        None,
    );
    let result = usecase.update_todo(update_view).await;

    // update() 내 fetch_one이 RowNotFound → ? 전파 → tx 롤백
    assert!(
        result.is_err(),
        "updating nonexistent todo must return Err: {result:?}"
    );
}
