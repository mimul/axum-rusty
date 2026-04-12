use crate::module::todo_module::TodoModule;
use crate::module::user_module::UserModule;
use common::config::ApplicationConfig;
use infra::db::Db;
use infra::repository::health_check::HealthCheckRepository;
use infra::repository::todo::status::TodoStatusRepository;
use infra::repository::todo::TodoRepository;
use infra::repository::user::UserRepository;
use std::sync::Arc;
use usecase::usecase::health_check::HealthCheckUseCase;

/// 전체 도메인 모듈을 조합하는 DI 루트.
///
/// 도메인별 모듈(`TodoModule`, `UserModule`)과
/// 인프라 관심사(`HealthCheckUseCase`)를 보유한다.
/// 새 도메인 추가 시 해당 Module과 Repository만 이곳에 추가하면 된다.
pub struct UseCaseModules {
    pub(crate) todo: TodoModule,
    pub(crate) user: UserModule,
    pub(crate) health_check: HealthCheckUseCase,
}

impl UseCaseModules {
    pub fn new(db: Db) -> Self {
        let pool = (*db.0).clone();

        let todo_repo = Arc::new(TodoRepository::new(pool.clone()));
        let todo_status_repo = Arc::new(TodoStatusRepository::new(pool.clone()));
        let user_repo = Arc::new(UserRepository::new(pool.clone()));

        Self {
            todo: TodoModule::new(pool.clone(), todo_repo, todo_status_repo),
            user: UserModule::new(pool.clone(), user_repo),
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
