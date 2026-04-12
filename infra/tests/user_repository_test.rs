mod common;

use common::db::setup_test_db;
use common::fixtures::fixture_new_user;
use domain::model::user::{NewUser, User};
use domain::model::Id;
use infra::repository::user::UserRepository;

/// insert → get_user (id 조회)
#[tokio::test]
async fn insert_user_stores_and_retrieves_by_id() {
    let pool = setup_test_db().await;
    let repo = UserRepository::new(pool.clone());
    let mut tx = pool.begin().await.unwrap();

    let inserted = repo
        .insert_tx(&mut tx, fixture_new_user("id_lookup"))
        .await
        .unwrap();
    let found = repo.get_user_tx(&mut tx, &inserted.id).await.unwrap();

    assert!(found.is_some(), "inserted user should be retrievable by id");
    assert_eq!(found.unwrap().id.value, inserted.id.value);
    tx.rollback().await.unwrap();
}

/// insert → get_user_by_username (username 조회)
#[tokio::test]
async fn insert_user_then_get_by_username_returns_user() {
    let pool = setup_test_db().await;
    let repo = UserRepository::new(pool.clone());
    let mut tx = pool.begin().await.unwrap();

    let new_user = fixture_new_user("by_username");
    let username = new_user.username.clone();
    let inserted = repo.insert_tx(&mut tx, new_user).await.unwrap();

    let found = repo
        .get_user_by_username_tx(&mut tx, &username)
        .await
        .unwrap();

    assert!(
        found.is_some(),
        "inserted user should be retrievable by username"
    );
    assert_eq!(found.unwrap().id.value, inserted.id.value);
    tx.rollback().await.unwrap();
}

/// 존재하지 않는 id → None 반환
#[tokio::test]
async fn get_user_with_nonexistent_id_returns_none() {
    let pool = setup_test_db().await;
    let repo = UserRepository::new(pool.clone());

    let id: Id<User> = Id::gen();
    let found = repo.get_user(&id).await.unwrap();

    assert!(found.is_none());
}

/// 존재하지 않는 username → None 반환
#[tokio::test]
async fn get_user_by_username_nonexistent_returns_none() {
    let pool = setup_test_db().await;
    let repo = UserRepository::new(pool.clone());

    let found = repo
        .get_user_by_username("no_such_user_xyz_99999")
        .await
        .unwrap();

    assert!(found.is_none());
}

/// 중복 username insert → unique 제약 에러
#[tokio::test]
async fn insert_duplicate_username_returns_error() {
    let pool = setup_test_db().await;
    let repo = UserRepository::new(pool.clone());
    let mut tx = pool.begin().await.unwrap();

    let first = fixture_new_user("dup_user");
    let username = first.username.clone();
    repo.insert_tx(&mut tx, first).await.unwrap();

    let second = NewUser::new(
        Id::gen(),
        username,
        "other_password".to_string(),
        "Other Fullname".to_string(),
    );
    let result = repo.insert_tx(&mut tx, second).await;

    assert!(result.is_err(), "inserting duplicate username should fail");
    tx.rollback().await.unwrap();
}
