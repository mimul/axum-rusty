use crate::module::todo_module::TodoModule;
use crate::module::user_module::UserModule;
use common::config::ApplicationConfig;
use infra::module::uow::PgUnitOfWorkFactory;
use infra::persistence::postgres::Db;
use infra::repository::health_check::HealthCheckRepository;
use std::sync::Arc;
use usecase::usecase::health_check::HealthCheckUseCase;

/// 전체 도메인 모듈을 조합하는 DI 루트.
///
/// 도메인별 모듈(`TodoModule`, `UserModule`)과
/// 인프라 관심사(`HealthCheckUseCase`)를 보유한다.
pub struct UseCaseModules {
    pub todo: TodoModule,
    pub user: UserModule,
    pub health_check: HealthCheckUseCase,
}

impl UseCaseModules {
    pub fn new(db: Db) -> Self {
        let pool = (*db.0).clone();
        let uow_factory = Arc::new(PgUnitOfWorkFactory::new(pool.clone()));
        Self {
            todo: TodoModule::new(uow_factory.clone()),
            user: UserModule::new(uow_factory),
            health_check: HealthCheckUseCase::new(HealthCheckRepository::new(pool)),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub modules: Arc<UseCaseModules>,
    pub config: Arc<ApplicationConfig>,
}

impl AppState {
    pub fn new(db: Db, config: ApplicationConfig) -> Self {
        let modules = Arc::new(UseCaseModules::new(db));
        let config = Arc::new(config);
        Self { modules, config }
    }
}
