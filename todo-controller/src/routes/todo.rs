use crate::context::api_response::ApiResponse;
use crate::context::api_version::ApiVersion;
use crate::context::errors::AppError;
use crate::context::validate::ValidatedRequest;
use crate::model::todo;
use crate::model::todo::{
    JsonCreateTodo, JsonTodo, JsonTodoList, JsonUpdateTodoContents, JsonUpsertTodoContents,
    TodoQuery,
};
use crate::module::{Modules, ModulesExt};
use axum::extract::{Path, Query, State};
use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use serde::de::Unexpected::Option;
use serde_json::{json, Value};
use std::sync::Arc;
use todo_domain::model::todo::Todo;
use todo_usecase::model::todo::status::TodoStatusView;
use todo_usecase::model::todo::TodoView;
use tracing::log::{error, info};

pub async fn error_handler (
    uri: Uri,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    Err(AppError::Error("abnormal uri".to_string()))
}

pub async fn get_todo(
    _: ApiVersion,
    Path((_v, id)): Path<(ApiVersion, String)>,
    modules: State<Arc<Modules>>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("get_todo: id={}", id);
    let resp = modules.todo_use_case().get_todo(id).await;
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

pub async fn find_todo(
    _: ApiVersion,
    Query(query): Query<TodoQuery>,
    modules: State<Arc<Modules>>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("find_todo: id={:?}", query);
    if query.status.is_none() {
        info!("status is none. id={:?}", query);
        return Err(AppError::Error("status is none".to_string()));
    }
    let resp = modules.todo_use_case().find_todo(query.into()).await;
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
                    message: "data is not found.".to_string(),
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

pub async fn create_todo(
    _: ApiVersion,
    modules: State<Arc<Modules>>,
    ValidatedRequest(source): ValidatedRequest<JsonCreateTodo>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("create_todo: {:?}", source);
    let resp = modules.todo_use_case().register_todo(source.into()).await;
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

pub async fn update_todo(
    _: ApiVersion,
    Path((_v, id)): Path<(ApiVersion, String)>,
    modules: State<Arc<Modules>>,
    ValidatedRequest(source): ValidatedRequest<JsonUpdateTodoContents>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("update_todo: {:?}", source);
    match source.validate(id) {
        Ok(todo) => {
            let resp = modules.todo_use_case().update_todo(todo).await;
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
        Err(errors) => Err(AppError::Error("invalid_request".to_string())),
    }
}

pub async fn upsert_todo(
    _: ApiVersion,
    Path((_v, id)): Path<(ApiVersion, String)>,
    modules: State<Arc<Modules>>,
    ValidatedRequest(source): ValidatedRequest<JsonUpsertTodoContents>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("upsert_todo: {:?}", source);
    let resp = modules
        .todo_use_case()
        .upsert_todo(source.to_view(id))
        .await;
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

pub async fn delete_todo(
    _: ApiVersion,
    Path((_v, id)): Path<(ApiVersion, String)>,
    modules: State<Arc<Modules>>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    info!("delete_todo: id={}", id);
    let resp = modules.todo_use_case().delete_todo(id).await;
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
