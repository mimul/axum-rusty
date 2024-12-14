use std::sync::Arc;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse};
use jsonwebtoken::decode;
use tracing::log::{error};
use usecase::model::user::UserView;
use crate::context::errors::AppError;
use crate::context::errors::AppError::InvalidJwt;
use crate::model::user::{TokenClaims};
use crate::module::{Modules, ModulesExt};

pub async fn auth (
    modules: State<Arc<Modules>>,
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

    match authorize_current_user(auth_header, &modules).await {
        Ok(current_user) => {
            req.extensions_mut().insert(current_user);
            return Ok(next.run(req).await)
        }
        Err(err) => {
            error!("error authorizing user: {:?}", err);
            return Err(InvalidJwt(err.to_string()))
        }
    }

    async fn authorize_current_user(auth_token: &str, modules: &Modules) -> Result<UserView, AppError> {
        let claims = decode::<TokenClaims>(
            auth_token,
            &jsonwebtoken::DecodingKey::from_secret(modules.constants.jwt_key.as_ref()),
            &jsonwebtoken::Validation::default(),
        );

        match claims {
            Ok(claims) => {
                let user_id = claims.claims.sub;
                let user_view = modules.user_use_case().get_user(user_id).await;
                match user_view {
                    Ok(user_view) => match user_view {
                        Some(uv) => {
                            Ok(uv.into())
                        }
                        None => {
                            Err(InvalidJwt("user not found".to_string()))
                        }
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