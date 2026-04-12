use std::sync::Arc;
use usecase::module::uow::UserUnitOfWorkFactory;
use usecase::usecase::user::UserUseCase;

/// User 도메인 모듈.
///
/// User 관련 유스케이스를 묶어 관리한다.
/// 도메인이 성장하면 이 구조체에 유스케이스를 추가한다.
pub struct UserModule {
    pub use_case: UserUseCase,
}

impl UserModule {
    pub fn new(uow_factory: Arc<dyn UserUnitOfWorkFactory>) -> Self {
        Self {
            use_case: UserUseCase::new(uow_factory),
        }
    }
}
