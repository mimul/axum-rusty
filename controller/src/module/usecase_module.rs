use common::config::ApplicationConfig;
use infra::module::uow::PgUnitOfWorkFactory;
use infra::persistence::postgres::Db;
use infra::repository::health_check::HealthCheckRepository;
use std::sync::Arc;
use usecase::usecase::health_check::HealthCheckUseCase;
use usecase::usecase::todo::TodoUseCase;
use usecase::usecase::user::UserUseCase;

pub struct UseCaseModules {
    pub user_use_case: UserUseCase,
    pub health_check_use_case: HealthCheckUseCase,
    pub todo_use_case: TodoUseCase,
}

impl UseCaseModules {
    pub fn new(db: Db) -> Self {
        let pool = (*db.0).clone();
        let uow_factory = Arc::new(PgUnitOfWorkFactory::new(pool.clone()));
        let user_use_case = UserUseCase::new(uow_factory.clone());
        let health_check_use_case = HealthCheckUseCase::new(HealthCheckRepository::new(pool));
        let todo_use_case = TodoUseCase::new(uow_factory);

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
