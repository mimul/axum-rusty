//! TodoUseCase 트랜잭션 롤백 통합 테스트
//!
//! 실행 방법:
//! ```
//! TEST_DATABASE_URL="postgresql://postgres:postgres@localhost:5432/todo_db" \
//!   cargo test -p usecase --test todo_usecase_integration_test -- --test-threads=1
//! ```
//!
//! `--test-threads=1` 필수:
//! `#[tokio::test]`는 테스트마다 독립 tokio 런타임을 생성한다.
//! 병렬 실행 시 각 테스트가 별도 커넥션 풀을 만들어 DB 커넥션이 고갈될 수 있다.

mod common;

use common::db::setup_test_db;
use domain::model::todo::NewTodo;
use domain::model::Id;
use domain::repository::todo::TodoRepository;
use infra::module::repo_module::RepositoriesModule;
use infra::persistence::postgres::Db;
use infra::repository::todo::TodoRepositoryImpl;
use std::sync::Arc;
use usecase::model::todo::{CreateTodo, UpdateTodoView, UpsertTodoView};
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
    assert!(result.is_err(), "invalid status_code must return Err, got: {result:?}");

    // Assert: DB에 변경 없음 (롤백 검증)
    let mut verify_tx = pool.begin().await.unwrap();
    let found = todo_repo.get(&inserted.id, &mut verify_tx).await.unwrap();
    verify_tx.rollback().await.unwrap();

    let found = found.expect("todo must still exist after rollback");
    assert_eq!(
        found.title, "Original Title",
        "title must be unchanged after transaction rollback"
    );

    // Cleanup: 테스트 데이터 삭제
    let mut cleanup_tx = pool.begin().await.unwrap();
    todo_repo
        .delete(&inserted.id, &mut cleanup_tx)
        .await
        .unwrap();
    cleanup_tx.commit().await.unwrap();
}

/// create_and_update_todo에서 create는 성공하고 update가 실패할 때
/// 단일 트랜잭션이 롤백되어 create된 todo도 DB에 반영되지 않음을 검증한다.
#[tokio::test]
async fn create_and_update_todo_when_update_fails_rolls_back_create() {
    let pool = setup_test_db().await;

    // Setup: update 대상 todo를 커밋하여 DB에 저장
    let todo_repo = TodoRepositoryImpl::new();
    let mut setup_tx = pool.begin().await.unwrap();
    let target_todo = todo_repo
        .insert(
            NewTodo::new(
                Id::gen(),
                "Update Target".to_string(),
                "Target Desc".to_string(),
            ),
            &mut setup_tx,
        )
        .await
        .unwrap();
    setup_tx.commit().await.unwrap();

    // Act: create는 성공하지만 update가 invalid status_code로 실패
    // → 단일 tx 전체가 롤백 → create된 todo도 DB에 없어야 함
    let db = Db(Arc::new(pool.clone()));
    let repos = Arc::new(RepositoriesModule::new());
    let usecase = TodoUseCase::new(db, repos);

    // 실행별 고유 title — panic 시 잔류해도 다음 실행 assertion에 영향 없음
    let unique_title = format!("__ROLLBACK_CREATE_TEST__{}", Id::gen().value);
    let create_source = CreateTodo::new(
        unique_title.clone(),
        "This todo must not persist".to_string(),
    );
    let update_source = UpdateTodoView::new(
        target_todo.id.value.to_string(),
        Some("Should Not Be Saved".to_string()),
        None,
        Some("INVALID_STATUS_THAT_DOES_NOT_EXIST".to_string()),
    );

    let result = usecase
        .create_and_update_todo(create_source, update_source)
        .await;

    // Assert: Err 반환 (update 실패)
    assert!(result.is_err(), "update failure must propagate as Err, got: {result:?}");

    // Assert: create도 롤백됨 — 고유 타이틀로 존재 여부 확인
    let mut verify_tx = pool.begin().await.unwrap();
    let all_todos = todo_repo
        .find(None, &mut verify_tx)
        .await
        .unwrap()
        .unwrap_or_default();
    verify_tx.rollback().await.unwrap();

    let rolled_back = all_todos
        .iter()
        .any(|t| t.title == unique_title);
    assert!(
        !rolled_back,
        "created todo must not exist in DB after transaction rollback"
    );

    // Assert: update 대상 todo는 변경 없음
    let mut verify_tx2 = pool.begin().await.unwrap();
    let target = todo_repo
        .get(&target_todo.id, &mut verify_tx2)
        .await
        .unwrap();
    verify_tx2.rollback().await.unwrap();

    let target = target.expect("update target must still exist after rollback");
    assert_eq!(
        target.title, "Update Target",
        "update target title must be unchanged after rollback"
    );

    // Cleanup
    let mut cleanup_tx = pool.begin().await.unwrap();
    todo_repo
        .delete(&target_todo.id, &mut cleanup_tx)
        .await
        .unwrap();
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
    let update_view =
        UpdateTodoView::new(nonexistent_id, Some("Ghost Title".to_string()), None, None);
    let result = usecase.update_todo(update_view).await;

    // update() 내 fetch_one이 RowNotFound → ? 전파 → tx 롤백
    assert!(
        result.is_err(),
        "updating nonexistent todo must return Err: {result:?}"
    );
}

/// create_and_update_todo에서 update_source.id가 잘못된 ULID 형식이면
/// insert 성공 후 try_into() 실패 → tx 롤백 → create된 todo도 DB에 없음을 검증한다.
///
/// 실패 지점:
///   insert (성공, tx에 데이터 존재)
///   → get_by_code 생략 (status_code = None)
///   → update_source.id.try_into()? 실패 (잘못된 ULID 형식)
///   → ? 전파 → tx drop → 자동 롤백
#[tokio::test]
async fn create_and_update_todo_with_invalid_id_format_rolls_back_create() {
    let pool = setup_test_db().await;
    let db = Db(Arc::new(pool.clone()));
    let repos = Arc::new(RepositoriesModule::new());
    let usecase = TodoUseCase::new(db, repos);

    // 실행별 고유 title — panic 시 잔류해도 다음 실행 assertion에 영향 없음
    let unique_title = format!("__ROLLBACK_INVALID_ID_TEST__{}", Id::gen().value);

    // Act: create는 유효, update id는 잘못된 형식 + status_code = None
    let create_source = CreateTodo::new(
        unique_title.clone(),
        "Must be rolled back".to_string(),
    );
    let update_source = UpdateTodoView::new(
        "NOT_A_VALID_ULID_FORMAT".to_string(),
        Some("Ghost Title".to_string()),
        None,
        None, // status_code = None → get_by_code 생략 → try_into()까지 진행
    );

    let result = usecase
        .create_and_update_todo(create_source, update_source)
        .await;

    // Assert: 에러 반환 (invalid ULID)
    assert!(result.is_err(), "invalid ID format must return Err, got: {result:?}");

    // Assert: insert된 todo가 DB에 없음 (롤백 검증)
    let todo_repo = TodoRepositoryImpl::new();
    let mut verify_tx = pool.begin().await.unwrap();
    let all = todo_repo
        .find(None, &mut verify_tx)
        .await
        .unwrap()
        .unwrap_or_default();
    verify_tx.rollback().await.unwrap();

    let rolled_back = all
        .iter()
        .any(|t| t.title == unique_title);
    assert!(
        !rolled_back,
        "created todo must not exist in DB after transaction rollback"
    );
}

/// upsert_todo에 존재하지 않는 status_code를 넘기면
/// get_by_code가 Err를 반환하고 트랜잭션이 롤백되어
/// upsert된 데이터가 DB에 반영되지 않음을 검증한다.
#[tokio::test]
async fn upsert_todo_with_invalid_status_rolls_back_transaction() {
    let pool = setup_test_db().await;
    let db = Db(Arc::new(pool.clone()));
    let repos = Arc::new(RepositoriesModule::new());
    let usecase = TodoUseCase::new(db, repos);

    let upsert_id = Id::<domain::model::todo::Todo>::gen().value.to_string();
    let upsert_source = UpsertTodoView::new(
        upsert_id.clone(),
        "Upsert Title".to_string(),
        "Upsert Desc".to_string(),
        "INVALID_STATUS_CODE".to_string(),
    );

    let result = usecase.upsert_todo(upsert_source).await;

    // Assert: 에러 반환 (invalid status_code)
    assert!(result.is_err(), "invalid status_code must return Err, got: {result:?}");

    // Assert: upsert된 todo가 DB에 없음 (롤백 검증)
    let todo_repo = TodoRepositoryImpl::new();
    let parsed_id: Id<domain::model::todo::Todo> = upsert_id.try_into().unwrap();
    let mut verify_tx = pool.begin().await.unwrap();
    let found = todo_repo.get(&parsed_id, &mut verify_tx).await.unwrap();
    verify_tx.rollback().await.unwrap();

    assert!(
        found.is_none(),
        "upserted todo must not exist in DB after transaction rollback"
    );
}
