use common::config::ApplicationConfig;
#[allow(unused_imports)]
use infra::db::Db;
#[allow(unused_imports)]
use infra::repository::health_check::HealthCheckRepository;
#[allow(unused_imports)]
use infra::repository::todo::status::TodoStatusRepository;
#[allow(unused_imports)]
use infra::repository::todo::TodoRepository;
#[allow(unused_imports)]
use infra::repository::user::UserRepository;
use shaku::module;
use std::sync::Arc;
#[allow(unused_imports)]
use usecase::usecase::health_check::HealthCheckUseCase;
#[allow(unused_imports)]
use usecase::usecase::todo::TodoUseCase;
#[allow(unused_imports)]
use usecase::usecase::user::UserUseCase;

// 새 도메인 추가 시:
// 1. infra에 Repository + `#[derive(Component)]`
// 2. usecase에 UseCase + `#[derive(Component)]`
// 3. 아래 components 목록에 등록
module! {
    pub AppModule {
        components = [
            Db,
            TodoRepository,
            TodoStatusRepository,
            UserRepository,
            HealthCheckRepository,
            TodoUseCase,
            UserUseCase,
            HealthCheckUseCase,
        ],
        providers = []
    }
}

/// Axum 공유 상태.
///
/// 핸들러에서 `state.module.resolve::<dyn IFooUseCase>()` 로 유스케이스를 주입받는다.
#[derive(Clone)]
pub struct AppState {
    pub module: Arc<AppModule>,
    pub config: Arc<ApplicationConfig>,
}

impl AppState {
    pub fn new(module: Arc<AppModule>, config: ApplicationConfig) -> Self {
        Self {
            module,
            config: Arc::new(config),
        }
    }
}
