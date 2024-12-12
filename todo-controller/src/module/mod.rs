use std::sync::Arc;
use todo_infra::modules::{RepositoriesModule, RepositoriesModuleExt};
use todo_infra::persistence::postgres::Db;
use todo_infra::repository::health_check::HealthCheckRepository;
use todo_usecase::usecase::health_check::HealthCheckUseCase;
use todo_usecase::usecase::todo::TodoUseCase;
use todo_usecase::usecase::user::UserUseCase;

pub struct Modules {
    user_use_case: UserUseCase<RepositoriesModule>,
    health_check_use_case: HealthCheckUseCase,
    todo_use_case: TodoUseCase<RepositoriesModule>,
}

pub trait ModulesExt {
    type RepositoriesModule: RepositoriesModuleExt;

    fn user_use_case(&self) -> &UserUseCase<Self::RepositoriesModule>;
    fn health_check_use_case(&self) -> &HealthCheckUseCase;
    fn todo_use_case(&self) -> &TodoUseCase<Self::RepositoriesModule>;
}

impl ModulesExt for Modules {
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

impl Modules {
    pub async fn new() -> Self {
        let db = Db::new().await;
        let repositories_module = Arc::new(RepositoriesModule::new(db.clone()));
        let user_use_case = UserUseCase::new(repositories_module.clone());
        let health_check_use_case = HealthCheckUseCase::new(HealthCheckRepository::new(db));
        let todo_use_case = TodoUseCase::new(repositories_module.clone());

        Self {
            user_use_case,
            health_check_use_case,
            todo_use_case,
        }
    }
}
