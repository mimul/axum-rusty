use crate::context::api_version::ApiVersion;
use crate::module::usecase_module::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use log::{debug, error};
use shaku::HasComponent;
use std::sync::Arc;
use usecase::usecase::health_check::IHealthCheckUseCase;

pub async fn hc(_: ApiVersion) -> impl IntoResponse {
    debug!("Access health check endpoint.");
    StatusCode::NO_CONTENT
}

pub async fn hc_postgres(
    _: ApiVersion,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let uc: Arc<dyn IHealthCheckUseCase> = state.module.resolve();
    uc.diagnose_db_conn()
        .await
        .map(|_| {
            debug!("Access postgres health check endpoint.");
            StatusCode::NO_CONTENT
        })
        .map_err(|err| {
            error!("{:?}", err);
            StatusCode::SERVICE_UNAVAILABLE
        })
}
