#![allow(dead_code)]

use domain::model::todo::NewTodo;
use domain::model::user::NewUser;
use domain::model::Id;

/// 고유한 suffix로 중복 username 충돌을 방지하는 NewUser 픽스처.
pub fn fixture_new_user(suffix: &str) -> NewUser {
    NewUser::new(
        Id::gen(),
        format!("testuser_{suffix}"),
        "hashed_password_test".to_string(),
        format!("Test Fullname {suffix}"),
    )
}

/// 기본값으로 채워진 NewTodo 픽스처.
pub fn fixture_new_todo() -> NewTodo {
    NewTodo::new(
        Id::gen(),
        "Test Todo Title".to_string(),
        "Test Todo Description".to_string(),
    )
}
