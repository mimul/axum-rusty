use crate::context::errors::AppError;
use crate::context::errors::AppError::InvalidJwt;
use crate::model::user::TokenClaims;
use crate::module::usecase_module::AppState;
use axum::extract::{Request, State};
use axum::{middleware::Next, response::IntoResponse};
use common::auth::webs::{get_auth_header, get_cookie_from_headers};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use log::{error, info};
use shaku::HasComponent;
use std::sync::Arc;
use usecase::model::user::UserView;
use usecase::usecase::user::IUserUseCase;

pub async fn auth(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    let access_token = get_cookie_from_headers("access_token", req.headers())
        .or_else(|| get_auth_header(req.headers()).map(|s| s.to_string()))
        .ok_or_else(|| InvalidJwt("auth_header not found".to_string()))?;
    info!("auth: access_token={:?}", access_token);
    log::logger().flush();

    let current_user = authorize_current_user(access_token, &state)
        .await
        .map_err(|err| {
            error!("error authorizing user: {:?}", err);
            InvalidJwt(err.to_string())
        })?;
    req.extensions_mut().insert(current_user);
    Ok(next.run(req).await)
}

async fn authorize_current_user(
    access_token: String,
    state: &AppState,
) -> Result<UserView, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    let claims = decode::<TokenClaims>(
        access_token.as_str(),
        &DecodingKey::from_secret(state.config.jwt_secret.as_ref()),
        &validation,
    )
    .map_err(|err| {
        error!("Error decoding token: {:?}", err);
        InvalidJwt(err.to_string())
    })?;

    let user_id = claims.claims.sub;
    let uc: Arc<dyn IUserUseCase> = state.module.resolve();
    uc.get_user(user_id)
        .await
        .map_err(|err| {
            error!("Unexpected error: {:?}", err);
            InvalidJwt(err.to_string())
        })?
        .ok_or_else(|| InvalidJwt("user not found".to_string()))
}
