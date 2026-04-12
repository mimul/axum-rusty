use common::config::ApplicationConfig;
use controller::module::usecase_module::AppState;
use controller::startup::startup;
use dotenvy::dotenv;
use infra::db::Db;
use log::info;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("common/src/config/log4rs.yaml", Default::default())?;
    dotenv().ok();
    //init_app();
    let config = ApplicationConfig::init();
    info!("main: config={:?}", config);
    let db = Db::new(config.clone()).await;
    let app_state = AppState::new(db.clone(), config.clone());
    startup(Arc::new(app_state)).await;
    Ok(())
}
