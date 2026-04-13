use common::config::ApplicationConfig;
use controller::module::usecase_module::{AppModule, AppState};
use controller::startup::startup;
use dotenvy::dotenv;
use infra::db::{create_pool, Db, DbParameters};
use log::info;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("common/src/config/log4rs.yaml", Default::default())?;
    dotenv().ok();
    let config = ApplicationConfig::init();
    info!("main: config={:?}", config);

    let pool = create_pool(&config).await?;
    let module = Arc::new(
        AppModule::builder()
            .with_component_parameters::<Db>(DbParameters { pool })
            .build(),
    );
    let app_state = AppState::new(module, config);
    startup(Arc::new(app_state)).await;
    Ok(())
}
