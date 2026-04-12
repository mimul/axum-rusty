use std::sync::Arc;
use usecase::module::uow::TodoUnitOfWorkFactory;
use usecase::usecase::todo::TodoUseCase;

/// Todo 도메인 모듈.
///
/// Todo 관련 유스케이스를 묶어 관리한다.
/// 도메인이 성장하면 이 구조체에 유스케이스를 추가한다.
pub struct TodoModule {
    pub use_case: TodoUseCase,
}

impl TodoModule {
    pub fn new(uow_factory: Arc<dyn TodoUnitOfWorkFactory>) -> Self {
        Self {
            use_case: TodoUseCase::new(uow_factory),
        }
    }
}
