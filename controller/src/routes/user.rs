use crate::context::api_response::ApiResponse;
use crate::context::api_version::ApiVersion;
use crate::context::errors::AppError;
use crate::context::validate::ValidatedRequest;
use crate::model::user::{JsonCreateUser, JsonLoginUser, JsonUser, TokenClaims, UserQuery};
use crate::module::{Modules, ModulesExt};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::log::{error, info};
use usecase::model::user::UserView;

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(create_user, get_user, get_user_by_username, login_user),
    components(schemas(JsonCreateUser, UserQuery, ApiResponse<Value>)),
    tags((name = "User"))
)]
pub struct UserOpenApi;

#[utoipa::path(
    post,
    path = "/v1/user/create",
    request_body(
        content = JsonCreateUser,
        content_type = "application/json"
    ),
    operation_id = stringify!(create_user),
    responses(
        (status = OK, description = "user created successfully", body = ApiResponse<Value>)
    ),
    tag = "User",
)]
pub async fn create_user(
    _: ApiVersion,
    modules: State<Arc<Modules>>,
    ValidatedRequest(source): ValidatedRequest<JsonCreateUser>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("create_user source={:?}", source);
    let resp = modules.user_use_case().create_user(source.into()).await;
    resp.map(|tv| {
        info!("created user: {}", tv.id);
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
        AppError::Error(err.to_string())
    })
}

#[utoipa::path(
    get,
    path = "/v1/user/{id}",
    operation_id = stringify!(get_user),
    responses(
        (status = OK, description = "Get one user successfully", body = ApiResponse<Value>)
    ),
    tag = "User",
)]
pub async fn get_user(
    _: ApiVersion,
    Path((_v, id)): Path<(ApiVersion, String)>,
    modules: State<Arc<Modules>>,
    Extension(current_user): Extension<UserView>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("get_user: id={}, current_user={:?}", id, current_user);
    let resp = modules.user_use_case().get_user(id).await;
    match resp {
        Ok(uv) => uv
            .map(|uv| {
                info!("found user `{:?}`.", uv);
                let json: JsonUser = uv.into();
                let response: ApiResponse<Value> = ApiResponse::<Value> {
                    result: true,
                    message: "success".to_string(),
                    data: Some(json!({
                        "userView": json,
                    })),
                };
                return (StatusCode::OK, Json(response));
            })
            .ok_or_else(|| {
                error!("user is not found.");
                AppError::Error("data not found".to_string())
            }),
        Err(err) => {
            error!("Unexpected error: {:?}", err);
            Err(AppError::Error(err.to_string()))
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
    tag = "User",
)]
pub async fn get_user_by_username(
    _: ApiVersion,
    Query(query): Query<UserQuery>,
    modules: State<Arc<Modules>>,
    Extension(current_user): Extension<UserView>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!(
        "get_user_by_username: param={:?}, current_user={:?}",
        query, current_user
    );
    if query.username.is_empty() {
        info!("get_user_by_username: username is empty. id={:?}", query);
        return Err(AppError::Error("username is empty".to_string()));
    }
    let user_view = modules
        .user_use_case()
        .get_user_by_username(query.into())
        .await;
    match user_view {
        Ok(user_view) => match user_view {
            Some(uv) => {
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
            Err(AppError::Error(err.to_string()))
        }
    }
}

#[utoipa::path(
    get,
    path = "/v1/user/login",
    params(JsonCreateUser),
    operation_id = stringify!(login_user),
    responses(
        (status = OK, description = "login one user successfully", body = ApiResponse<Value>)
    ),
    tag = "User",
)]
pub async fn login_user(
    _: ApiVersion,
    modules: State<Arc<Modules>>,
    ValidatedRequest(source): ValidatedRequest<JsonLoginUser>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("login_user {:?}", source);
    let user_view = modules.user_use_case().login_user(source.into()).await;
    match user_view {
        Ok(user_view) => match user_view {
            uv => {
                let exp = (Utc::now()
                    + Duration::minutes(modules.constants.jwt_duration.parse().unwrap()))
                .timestamp() as usize;
                let claims: TokenClaims = TokenClaims {
                    sub: uv.id.clone().to_string(),
                    username: uv.username.clone(),
                    exp,
                };

                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(modules.constants.jwt_key.as_ref()),
                )
                .map_err(|_| AppError::InvalidJwt("token encoding error".to_string()));
                let json_user: JsonUser = uv.into();
                let response: ApiResponse<Value> = ApiResponse::<Value> {
                    result: true,
                    message: "success.".to_string(),
                    data: Some(json!({
                        "userView": json_user,
                        "token": token?,
                    })),
                };
                Ok((StatusCode::OK, Json(response)))
            }
        },
        Err(err) => {
            error!("Unexpected error: {:?}", err);
            Err(AppError::Error(err.to_string()))
        }
    }
}
