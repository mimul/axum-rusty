use crate::context::errors::AppError;
use crate::context::errors::AppError::InvalidJwt;
use crate::model::user::TokenClaims;
use crate::module::{AppState, ModulesExt};
use axum::{middleware::Next, response::IntoResponse};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::sync::Arc;
use axum::extract::{Request, State};
use log::{error, info};
use usecase::model::user::UserView;
use crate::context::webs::{get_auth_header, get_cookie_from_headers};

pub async fn auth(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    let access_token = get_cookie_from_headers("access_token", req.headers())
        .unwrap_or_else(|| {
            get_auth_header(req.headers()).unwrap().to_string()
        });
    info!("auth: access_token={:?}", access_token);
    log::logger().flush();
    if access_token.is_empty() {
        return Err(InvalidJwt("auth_header not found".to_string()));
    }

    match authorize_current_user(access_token, &state).await {
        Ok(current_user) => {
            req.extensions_mut().insert(current_user);
            return Ok(next.run(req).await)
        }
        Err(err) => {
            error!("error authorizing user: {:?}", err);
            return Err(InvalidJwt(err.to_string()));
        }
    }

    async fn authorize_current_user(
        access_token: String,
        state: &AppState,
    ) -> Result<UserView, AppError> {
        let claims = decode::<TokenClaims>(
            access_token.as_str(),
            &DecodingKey::from_secret(state.config.jwt_secret.as_ref()),
            &Validation::default(),
        );

        match claims {
            Ok(claims) => {
                let user_id = claims.claims.sub;
                let user_view = state.modules.user_use_case().get_user(user_id).await;
                match user_view {
                    Ok(user_view) => match user_view {
                        Some(uv) => Ok(uv.into()),
                        None => Err(InvalidJwt("user not found".to_string())),
                    },
                    Err(err) => {
                        error!("Unexpected error: {:?}", err);
                        Err(InvalidJwt(err.to_string()))
                    }
                }
            }
            Err(err) => {
                error!("Error decoding token: {:?}", err);
                Err(InvalidJwt(err.to_string()))
            }
        }
    }
}
