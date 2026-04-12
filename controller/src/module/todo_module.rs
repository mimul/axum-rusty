use infra::repository::todo::status::TodoStatusRepository;
use infra::repository::todo::TodoRepository;
use sqlx::PgPool;
use std::sync::Arc;
use usecase::usecase::todo::TodoUseCase;

/// Todo 도메인 모듈.
///
/// Todo 관련 유스케이스를 묶어 관리한다.
pub struct TodoModule {
    pub use_case: TodoUseCase,
}

impl TodoModule {
    pub fn new(
        pool: PgPool,
        todo_repo: Arc<TodoRepository>,
        todo_status_repo: Arc<TodoStatusRepository>,
    ) -> Self {
        Self {
            use_case: TodoUseCase::new(pool, todo_repo, todo_status_repo),
        }
    }
}
