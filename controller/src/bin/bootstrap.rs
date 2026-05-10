use common::config::ApplicationConfig;
use controller::module::usecase_module::{AppModule, AppState};
use controller::startup::startup;
use dotenvy::dotenv;
use infra::db::{create_pool, Db, DbParameters};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    dotenv().ok();
    let config = ApplicationConfig::try_init()?;
    info!(debug = %config.debug, allowed_origin = %config.allowed_origin, "server starting");

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
