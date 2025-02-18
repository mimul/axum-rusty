use crate::context::api_response::ApiResponse;
use crate::context::api_version::ApiVersion;
use crate::context::errors::AppError;
use crate::context::validate::ValidatedRequest;
use crate::model::todo::{
    JsonCreateTodo, JsonTodo, JsonTodoList, JsonUpdateTodoContents, JsonUpsertTodoContents,
    TodoQuery,
};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;
use log::{error, info};
use crate::module::usecase_module::{AppState, UseCaseModulesExt};

#[utoipa::path(
    get,
    path = "/v1/todo/{id}",
    operation_id = stringify!(get_todo),
    responses(
        (status = OK, description = "Get one todo successfully", body = ApiResponse<Value>)
    ),
    security(
        ("Authorization" = [])
    ),
    tag = "todo",
)]
pub async fn get_todo(
    _: ApiVersion,
    Path((_v, id)): Path<(ApiVersion, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("get_todo: id={}", id);
    let resp = state.modules.todo_use_case().get_todo(id).await;
    match resp {
        Ok(tv) => tv
            .map(|tv| {
                info!("found todo `{}`.", tv.id);
                let json: JsonTodo = tv.into();
                let response: ApiResponse<Value> = ApiResponse::<Value> {
                    result: true,
                    message: "success".to_string(),
                    data: Some(json!({
                        "todoView": json,
                    })),
                };
                (StatusCode::OK, Json(response))
            })
            .ok_or_else(|| {
                error!("todo is not found.");
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
    path = "/v1/todo",
    params(TodoQuery),
    operation_id = stringify!(find_todo),
    responses(
        (status = OK, description = "find all todos successfully", body = ApiResponse<Value>)
    ),
    security(
        ("Authorization" = [])
    ),
    tag = "todo",
)]
pub async fn find_todo(
    _: ApiVersion,
    Query(query): Query<TodoQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("find_todo: param={:?}", query);
    if query.status.is_none() {
        info!("status is none. id={:?}", query);
        return Err(AppError::Error("status is none".to_string()));
    }
    let resp = state.modules.todo_use_case().find_todo(query.into()).await;
    match resp {
        Ok(tv_list) => match tv_list {
            Some(tv) => {
                let todos = tv.into_iter().map(|t| t.into()).collect();
                let json = JsonTodoList::new(todos);
                let response: ApiResponse<Value> = ApiResponse::<Value> {
                    result: true,
                    message: "success".to_string(),
                    data: Some(json!({
                        "todoView": json,
                    })),
                };
                Ok((StatusCode::OK, Json(response)))
            }
            None => {
                let json = JsonTodoList::new(vec![]);
                let response: ApiResponse<Value> = ApiResponse::<Value> {
                    result: true,
                    message: "todo not found.".to_string(),
                    data: Some(json!({
                        "todoView": json,
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

#[utoipa::path(
    post,
    path = "/v1/todo",
    request_body(
        content = JsonCreateTodo,
        content_type = "application/json"
    ),
    operation_id = stringify!(create_todo),
    responses(
        (status = OK, description = "todo created successfully", body = ApiResponse<Value>)
    ),
    security(
        ("Authorization" = [])
    ),
    tag = "todo",
)]
pub async fn create_todo(
    _: ApiVersion,
    State(state): State<Arc<AppState>>,
    ValidatedRequest(source): ValidatedRequest<JsonCreateTodo>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("create_todo: {:?}", source);
    let resp = state.modules.todo_use_case().create_todo(source.into()).await;
    resp.map(|tv| {
        info!("created todo: {}", tv.id);
        let json: JsonTodo = tv.into();
        let response: ApiResponse<Value> = ApiResponse::<Value> {
            result: true,
            message: "success".to_string(),
            data: Some(json!({
                "todoView": json,
            })),
        };
        (StatusCode::OK, Json(response))
    })
    .map_err(|err| {
        error!("{:?}", err);
        AppError::Error("server_error".to_string())
    })
}

#[utoipa::path(
    patch,
    path = "/v1/todo/{id}",
    request_body(
        content = JsonUpdateTodoContents,
        content_type = "application/json"
    ),
    operation_id = stringify!(update_todo),
    responses(
        (status = OK, description = "Todo item updated successfully", body = ApiResponse<Value>)
    ),
    security(
        ("Authorization" = [])
    ),
    tag = "todo",
)]
pub async fn update_todo(
    _: ApiVersion,
    Path((_v, id)): Path<(ApiVersion, String)>,
    State(state): State<Arc<AppState>>,
    ValidatedRequest(source): ValidatedRequest<JsonUpdateTodoContents>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("update_todo: {:?}", source);
    match source.validate(id) {
        Ok(todo) => {
            let resp = state.modules.todo_use_case().update_todo(todo).await;
            resp.map(|tv| {
                info!("updated todo {}", tv.id);
                let json: JsonTodo = tv.into();
                let response: ApiResponse<Value> = ApiResponse::<Value> {
                    result: true,
                    message: "success".to_string(),
                    data: Some(json!({
                        "todoView": json,
                    })),
                };
                (StatusCode::OK, Json(response))
            })
            .map_err(|err| {
                error!("{:?}", err);
                AppError::Error(err.to_string())
            })
        }
        Err(errors) => Err(AppError::Error(
            errors
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(" or "),
        )),
    }
}

#[utoipa::path(
    put,
    path = "/v1/todo/{id}",
    request_body(
        content = JsonUpsertTodoContents,
        content_type = "application/json"
    ),
    operation_id = stringify!(upsert_todo),
    responses(
        (status = OK, description = "Todo item upserted successfully", body = ApiResponse<Value>)
    ),
    security(
        ("Authorization" = [])
    ),
    tag = "todo",
)]
pub async fn upsert_todo(
    _: ApiVersion,
    Path((_v, id)): Path<(ApiVersion, String)>,
    State(state): State<Arc<AppState>>,
    ValidatedRequest(source): ValidatedRequest<JsonUpsertTodoContents>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("upsert_todo: {:?}", source);
    let resp = state.modules.todo_use_case().upsert_todo(source.to_view(id)).await;
    resp.map(|tv| {
        info!("created or updated todo {}", tv.id);
        let json: JsonTodo = tv.into();
        let response: ApiResponse<Value> = ApiResponse::<Value> {
            result: true,
            message: "success".to_string(),
            data: Some(json!({
                "todoView": json,
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
    delete,
    path = "/v1/todo/{id}",
    operation_id = stringify!(delete_todo),
    responses(
        (status = OK, description = "Todo item created successfully", body = ApiResponse<Value>)
    ),
    security(
        ("Authorization" = [])
    ),
    tag = "todo",
)]
pub async fn delete_todo(
    _: ApiVersion,
    Path((_v, id)): Path<(ApiVersion, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("delete_todo: id={}", id);
    let resp = state.modules.todo_use_case().delete_todo(id).await;
    match resp {
        Ok(tv) => tv
            .map(|tv| {
                info!("Deleted todo `{}`.", tv.id);
                let json: JsonTodo = tv.into();
                let response: ApiResponse<Value> = ApiResponse::<Value> {
                    result: true,
                    message: "success".to_string(),
                    data: Some(json!({
                        "todoView": json,
                    })),
                };
                (StatusCode::OK, Json(response))
            })
            .ok_or_else(|| {
                error!("todo is not found.");
                AppError::Error("data not found".to_string())
            }),
        Err(err) => {
            error!("Unexpected error: {:?}", err);
            Err(AppError::Error(err.to_string()))
        }
    }
}
