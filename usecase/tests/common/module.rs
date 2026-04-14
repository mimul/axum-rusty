#![allow(unused_imports)]
use infra::db::{Db, DbParameters};
use infra::repository::todo::status::TodoStatusRepository;
use infra::repository::todo::TodoRepository;
use infra::repository::user::UserRepository;
use shaku::module;
use std::sync::Arc;
use usecase::usecase::todo::TodoUseCase;
use usecase::usecase::user::UserUseCase;

module! {
    pub UsecaseTestModule {
        components = [
            Db,
            TodoRepository,
            TodoStatusRepository,
            TodoUseCase,
            UserRepository,
            UserUseCase,
        ],
        providers = []
    }
}

pub fn build_usecase_test_module(pool: sqlx::PgPool) -> Arc<UsecaseTestModule> {
    Arc::new(
        UsecaseTestModule::builder()
            .with_component_parameters::<Db>(DbParameters { pool })
            .build(),
    )
}
