use infra::module::repo_module::RepositoriesModule;
use infra::persistence::postgres::Db;
use infra::repository::health_check::HealthCheckRepository;
use std::sync::Arc;
use usecase::module::repos::RepositoriesModuleExt;
use usecase::usecase::health_check::HealthCheckUseCase;
use usecase::usecase::todo::TodoUseCase;
use usecase::usecase::user::UserUseCase;
use common::config::ApplicationConfig;

pub struct UseCaseModules {
    user_use_case: UserUseCase<RepositoriesModule>,
    health_check_use_case: HealthCheckUseCase,
    todo_use_case: TodoUseCase<RepositoriesModule>,
}

pub trait UseCaseModulesExt {
    type RepositoriesModule: RepositoriesModuleExt;
    fn user_use_case(&self) -> &UserUseCase<Self::RepositoriesModule>;
    fn health_check_use_case(&self) -> &HealthCheckUseCase;
    fn todo_use_case(&self) -> &TodoUseCase<Self::RepositoriesModule>;
}

impl UseCaseModulesExt for UseCaseModules {
    type RepositoriesModule = RepositoriesModule;
    fn user_use_case(&self) -> &UserUseCase<Self::RepositoriesModule> {
        &self.user_use_case
    }
    fn health_check_use_case(&self) -> &HealthCheckUseCase {
        &self.health_check_use_case
    }
    fn todo_use_case(&self) -> &TodoUseCase<Self::RepositoriesModule> {
        &self.todo_use_case
    }
}

impl UseCaseModules {
    pub fn new(db: Db) -> Self {
        let pool = (*db.0).clone();
        let repositories_module = Arc::new(RepositoriesModule::new());
        let user_use_case = UserUseCase::new(pool.clone(), repositories_module.clone());
        let health_check_use_case = HealthCheckUseCase::new(HealthCheckRepository::new(db));
        let todo_use_case = TodoUseCase::new(pool, repositories_module);

        Self {
            user_use_case,
            health_check_use_case,
            todo_use_case,
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
