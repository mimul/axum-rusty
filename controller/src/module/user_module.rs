use infra::repository::user::PgUserRepository;
use sqlx::PgPool;
use std::sync::Arc;
use usecase::usecase::user::UserUseCase;

/// User 도메인 모듈.
///
/// User 관련 유스케이스를 묶어 관리한다.
pub struct UserModule {
    pub use_case: UserUseCase,
}

impl UserModule {
    pub fn new(pool: PgPool, user_repo: Arc<PgUserRepository>) -> Self {
        Self {
            use_case: UserUseCase::new(pool, user_repo),
        }
    }
}
