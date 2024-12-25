use crate::context::errors::AppError;
use crate::context::errors::AppError::InvalidJwt;
use crate::model::user::TokenClaims;
use crate::module::{AppState, ModulesExt};
use axum::{extract::State, middleware::Next, response::IntoResponse};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::sync::Arc;
use axum::extract::Request;
use tracing::log::error;
use usecase::model::user::UserView;

pub async fn auth(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                header.strip_prefix("Bearer ")
            } else {
                error!("auth_header not found");
                None
            }
        });
    let auth_header = match auth_header {
        Some(header) => header,
        None => return Err(InvalidJwt("auth_header not found".to_string())),
    };

    match authorize_current_user(auth_header, &state).await {
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
        auth_token: &str,
        //modules: &Modules,
        state: &AppState,
    ) -> Result<UserView, AppError> {
        let claims = decode::<TokenClaims>(
            auth_token,
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
