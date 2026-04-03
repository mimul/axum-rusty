mod common;

use common::db::setup_test_db;
use common::fixtures::fixture_new_user;
use domain::model::user::{NewUser, User};
use domain::model::Id;
use domain::repository::user::UserRepository;
use infra::repository::user::UserRepositoryImpl;

/// insert → get_user (id 조회)
#[tokio::test]
async fn insert_user_stores_and_retrieves_by_id() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = UserRepositoryImpl::new();

    let inserted = repo.insert(fixture_new_user("id_lookup"), &mut tx).await.unwrap();
    let found = repo.get_user(&inserted.id, &mut tx).await.unwrap();

    assert!(found.is_some(), "inserted user should be retrievable by id");
    assert_eq!(found.unwrap().id.value, inserted.id.value);
    tx.rollback().await.unwrap();
}

/// insert → get_user_by_username (username 조회)
#[tokio::test]
async fn insert_user_then_get_by_username_returns_user() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = UserRepositoryImpl::new();

    let new_user = fixture_new_user("by_username");
    let username = new_user.username.clone();
    let inserted = repo.insert(new_user, &mut tx).await.unwrap();

    let found = repo.get_user_by_username(&username, &mut tx).await.unwrap();

    assert!(found.is_some(), "inserted user should be retrievable by username");
    assert_eq!(found.unwrap().id.value, inserted.id.value);
    tx.rollback().await.unwrap();
}

/// 존재하지 않는 id → None 반환
#[tokio::test]
async fn get_user_with_nonexistent_id_returns_none() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = UserRepositoryImpl::new();

    let id: Id<User> = Id::gen();
    let found = repo.get_user(&id, &mut tx).await.unwrap();

    assert!(found.is_none());
    tx.rollback().await.unwrap();
}

/// 존재하지 않는 username → None 반환
#[tokio::test]
async fn get_user_by_username_nonexistent_returns_none() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = UserRepositoryImpl::new();

    let found = repo.get_user_by_username("no_such_user_xyz_99999", &mut tx).await.unwrap();

    assert!(found.is_none());
    tx.rollback().await.unwrap();
}

/// 중복 username insert → unique 제약 에러
#[tokio::test]
async fn insert_duplicate_username_returns_error() {
    let pool = setup_test_db().await;
    let mut tx = pool.begin().await.unwrap();
    let repo = UserRepositoryImpl::new();

    let first = fixture_new_user("dup_user");
    let username = first.username.clone();
    let _ = repo.insert(first, &mut tx).await.unwrap();

    let second = NewUser::new(
        Id::gen(),
        username,
        "other_password".to_string(),
        "Other Fullname".to_string(),
    );
    let result = repo.insert(second, &mut tx).await;

    assert!(result.is_err(), "inserting duplicate username should fail");
    tx.rollback().await.unwrap();
}
