#![allow(unused_imports)]
use infra::db::{Db, DbParameters};
use infra::repository::health_check::HealthCheckRepository;
use infra::repository::todo::status::TodoStatusRepository;
use infra::repository::todo::TodoRepository;
use infra::repository::user::UserRepository;
use shaku::module;
use std::sync::Arc;

module! {
    pub InfraTestModule {
        components = [
            Db,
            TodoRepository,
            TodoStatusRepository,
            UserRepository,
            HealthCheckRepository,
        ],
        providers = []
    }
}

pub fn build_test_module(pool: sqlx::PgPool) -> Arc<InfraTestModule> {
    Arc::new(
        InfraTestModule::builder()
            .with_component_parameters::<Db>(DbParameters { pool })
            .build(),
    )
}
