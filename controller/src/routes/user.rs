use crate::context::api_response::ApiResponse;
use crate::context::api_version::ApiVersion;
use crate::context::errors::AppError;
use crate::context::validate::ValidatedRequest;
use crate::model::user::{JsonCreateUser, JsonLoginUser, JsonUser, TokenClaims, UserQuery};
use crate::module::usecase_module::AppState;
use axum::extract::{Path, Query, State};
use axum::http::{header, Response, StatusCode};
use axum::{Extension, Json};
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use log::{error, info};
use serde_json::{json, Value};
use shaku::HasComponent;
use std::sync::Arc;
use usecase::model::user::UserView;
use usecase::usecase::user::IUserUseCase;

/// JWT 토큰을 생성한다.
fn generate_jwt_token(
    user_id: &str,
    username: &str,
    jwt_secret: &str,
    jwt_duration: i64,
) -> Result<String, AppError> {
    let now = Utc::now();
    let claims = TokenClaims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: (now + Duration::minutes(jwt_duration)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(|e| AppError::Error(format!("token encoding failed: {e}")))
}

#[utoipa::path(
    post,
    path = "/v1/auth/create",
    request_body(
        content = JsonCreateUser,
        content_type = "application/json"
    ),
    operation_id = stringify!(create_user),
    responses(
        (status = OK, description = "user created successfully", body = ApiResponse<Value>)
    ),
    tag = "user",
)]
pub async fn create_user(
    _: ApiVersion,
    State(state): State<Arc<AppState>>,
    ValidatedRequest(source): ValidatedRequest<JsonCreateUser>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("create_user request param={:?}", source);
    let uc: Arc<dyn IUserUseCase> = state.module.resolve();
    let resp = uc.create_user(source.into()).await;
    resp.map(|tv| {
        info!("create_user: response user: {}", tv.id);
        let json: JsonUser = tv.into();
        let response: ApiResponse<Value> = ApiResponse::<Value> {
            result: true,
            message: "success".to_string(),
            data: Some(json!({
                "userView": json,
            })),
        };
        (StatusCode::OK, Json(response))
    })
    .map_err(|err| {
        error!("{:?}", err);
        AppError::Error("서버 오류가 발생했습니다".to_string())
    })
}

#[utoipa::path(
    get,
    path = "/v1/user/{id}",
    operation_id = stringify!(get_user),
    responses(
        (status = OK, description = "Get one user successfully", body = ApiResponse<Value>)
    ),
    security(
        ("Authorization" = [])
    ),
    tag = "user",
)]
pub async fn get_user(
    _: ApiVersion,
    Path((_v, id)): Path<(ApiVersion, String)>,
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<UserView>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!(
        "get_user: request param id={}, current_user={:?}",
        id, current_user
    );
    let uc: Arc<dyn IUserUseCase> = state.module.resolve();
    let resp = uc.get_user(id).await;
    match resp {
        Ok(uv) => uv
            .map(|uv| {
                info!("get_user: response user={:?}.", uv);
                let json: JsonUser = uv.into();
                let response: ApiResponse<Value> = ApiResponse::<Value> {
                    result: true,
                    message: "success".to_string(),
                    data: Some(json!({
                        "userView": json,
                    })),
                };
                (StatusCode::OK, Json(response))
            })
            .ok_or_else(|| {
                error!("user is not found.");
                AppError::Error("data not found".to_string())
            }),
        Err(err) => {
            error!("Unexpected error: {:?}", err);
            Err(AppError::Error("서버 오류가 발생했습니다".to_string()))
        }
    }
}

#[utoipa::path(
    get,
    path = "/v1/user",
    params(UserQuery),
    operation_id = stringify!(get_user_by_username),
    responses(
        (status = OK, description = "Get one user successfully", body = ApiResponse<Value>)
    ),
    security(
        ("Authorization" = [])
    ),
    tag = "user",
)]
pub async fn get_user_by_username(
    _: ApiVersion,
    Query(query): Query<UserQuery>,
    State(state): State<Arc<AppState>>,
    Extension(current_user): Extension<UserView>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!(
        "get_user_by_username: request param={:?}, current_user={:?}",
        query, current_user
    );
    if query.username.is_empty() {
        info!("get_user_by_username: username is empty. id={:?}", query);
        return Err(AppError::Error("username is empty".to_string()));
    }
    let uc: Arc<dyn IUserUseCase> = state.module.resolve();
    let user_view = uc.get_user_by_username(query.into()).await;
    match user_view {
        Ok(user_view) => match user_view {
            Some(uv) => {
                info!("get_user_by_username: response user `{:?}`.", uv);
                let json: JsonUser = uv.into();
                let response: ApiResponse<Value> = ApiResponse::<Value> {
                    result: true,
                    message: "success".to_string(),
                    data: Some(json!({
                        "userView": json,
                    })),
                };
                Ok((StatusCode::OK, Json(response)))
            }
            None => {
                let response: ApiResponse<Value> = ApiResponse::<Value> {
                    result: true,
                    message: "user not found.".to_string(),
                    data: None,
                };
                Ok((StatusCode::OK, Json(response)))
            }
        },
        Err(err) => {
            error!("Unexpected error: {:?}", err);
            Err(AppError::Error("서버 오류가 발생했습니다".to_string()))
        }
    }
}

#[utoipa::path(
    post,
    path = "/v1/auth/login",
    request_body(
        content = JsonLoginUser,
        content_type = "application/json"
    ),
    operation_id = stringify!(login_user),
    responses(
        (status = OK, description = "login one user successfully", body = ApiResponse<Value>)
    ),
    tag = "user",
)]
pub async fn login_user(
    _: ApiVersion,
    State(state): State<Arc<AppState>>,
    ValidatedRequest(source): ValidatedRequest<JsonLoginUser>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("login_user: request param={:?}", source);
    let uc: Arc<dyn IUserUseCase> = state.module.resolve();
    let user_view = uc.login_user(source.into()).await;
    match user_view {
        Ok(uv) => {
            info!("login_user: response user `{:?}`.", uv);
            let jwt_duration = state
                .config
                .jwt_duration
                .parse::<i64>()
                .map_err(|e| AppError::Error(format!("jwt_duration 설정 오류: {e}")))?;
            let token =
                generate_jwt_token(&uv.id, &uv.username, &state.config.jwt_secret, jwt_duration)?;
            let cookie = Cookie::build("token", token.to_owned())
                .path("/")
                .max_age(time::Duration::hours(state.config.jwt_max_age.to_owned()))
                .same_site(SameSite::Lax)
                .http_only(true)
                .finish();
            let mut response = Response::new(json!({"status": "success"}).to_string());
            response.headers_mut().insert(
                header::SET_COOKIE,
                cookie
                    .to_string()
                    .parse()
                    .map_err(|e| AppError::Error(format!("cookie header parse failed: {e}")))?,
            );
            let json_user: JsonUser = uv.into();
            let response: ApiResponse<Value> = ApiResponse::<Value> {
                result: true,
                message: "success.".to_string(),
                data: Some(json!({
                    "userView": json_user,
                    "token": token,
                })),
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(err) => {
            error!("Unexpected error: {:?}", err);
            Err(AppError::Error("서버 오류가 발생했습니다".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_jwt_token_with_valid_inputs_returns_token() {
        let result = generate_jwt_token("user123", "alice", "secret_key_for_test", 60);
        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn generate_jwt_token_produces_three_part_jwt() {
        let token = generate_jwt_token("user123", "alice", "secret_key_for_test", 60).unwrap();
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
    }
}
