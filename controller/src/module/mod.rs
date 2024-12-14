use infra::modules::{RepositoriesModule, RepositoriesModuleExt};
use infra::persistence::postgres::Db;
use infra::repository::health_check::HealthCheckRepository;
use std::env;
use std::sync::Arc;
use usecase::usecase::health_check::HealthCheckUseCase;
use usecase::usecase::todo::TodoUseCase;
use usecase::usecase::user::UserUseCase;

pub struct Constants {
    pub jwt_key: String,
    pub allowed_origin: String,
    pub jwt_duration: String,
}

impl Constants {
    pub async fn new() -> Self {
        let jwt_key = env::var("JWT_KEY").unwrap_or_else(|_| panic!("JWT_KEY must be set!"));
        let allowed_origin =
            env::var("ALLOWED_ORIGIN").unwrap_or_else(|_| panic!("ALLOWED_ORIGIN must be set!"));
        let jwt_duration = env::var("JWT_DURATION_MINUTES")
            .unwrap_or_else(|_| panic!("JWT_DURATION_MINUTES must be set!"));

        Self {
            jwt_key,
            allowed_origin,
            jwt_duration,
        }
    }
}

pub struct Modules {
    pub(crate) constants: Constants,
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
        let constants = Constants::new().await;
        let repositories_module = Arc::new(RepositoriesModule::new(db.clone()));
        let user_use_case = UserUseCase::new(repositories_module.clone());
        let health_check_use_case = HealthCheckUseCase::new(HealthCheckRepository::new(db));
        let todo_use_case = TodoUseCase::new(repositories_module.clone());

        Self {
            constants,
            user_use_case,
            health_check_use_case,
            todo_use_case,
        }
    }
}
