use crate::context::api_version::ApiVersion;
use crate::context::validate::ValidatedRequest;
use crate::model::todo::{
    JsonCreateTodo, JsonTodo, JsonTodoList, JsonUpdateTodoContents, JsonUpsertTodoContents,
    TodoQuery,
};
use crate::module::{Modules, ModulesExt};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use std::sync::Arc;
use serde::de::Unexpected::Option;
use tracing::log::{error, info};
use todo_usecase::model::todo::TodoView;
use crate::context::api_response::{ApiResponse};
use crate::context::errors::AppError;
use serde_json::{json, Value};
use todo_usecase::model::todo::status::TodoStatusView;
use todo_domain::model::todo::Todo;
use crate::model::todo;

pub async fn get_todo(
    _: ApiVersion,
    Path((_v, id)): Path<(ApiVersion, String)>,
    modules: State<Arc<Modules>>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    let resp = modules.todo_use_case().get_todo(id).await;
    match resp {
        Ok(tv) => tv.map(|tv| {
            info!("Found todo `{}`.", tv.id);
            let json: JsonTodo = tv.into();
            let response: ApiResponse<Value> = ApiResponse::<Value> {
                success: true,
                message: "success".to_string(),
                data: Some(json!({
                    "todoView": json,
                })),
            };
            (StatusCode::OK, Json(response))
        }).ok_or_else(|| {
            error!("Todo is not found.");
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
    let resp = modules.todo_use_case().find_todo(query.into()).await;
    match resp {
        Ok(tv_list) => match tv_list {
            Some(tv) => {
                let todos = tv.into_iter().map(|t| t.into()).collect();
                let json = JsonTodoList::new(todos);
                let response: ApiResponse<Value> = ApiResponse::<Value> {
                    success: true,
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
                    success: true,
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
    let resp = modules.todo_use_case().register_todo(source.into()).await;
    resp.map(|tv| {
        info!("Created todo: {}", tv.id);
        let json: JsonTodo = tv.into();
        let response: ApiResponse<Value> = ApiResponse::<Value> {
            success: true,
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
    match source.validate(id) {
        Ok(todo) => {
            let resp = modules.todo_use_case().update_todo(todo).await;
            resp.map(|tv| {
                info!("Updated todo {}", tv.id);
                let json: JsonTodo = tv.into();
                let response: ApiResponse<Value> = ApiResponse::<Value> {
                    success: true,
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
        Err(errors) => {
            Err(AppError::Error("invalid_request".to_string()))
        }
    }
}

pub async fn upsert_todo(
    _: ApiVersion,
    Path((_v, id)): Path<(ApiVersion, String)>,
    modules: State<Arc<Modules>>,
    ValidatedRequest(source): ValidatedRequest<JsonUpsertTodoContents>,
) -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    let resp = modules
        .todo_use_case()
        .upsert_todo(source.to_view(id))
        .await;
    resp.map(|tv| {
        info!("Created or Updated todo {}", tv.id);
        let json: JsonTodo = tv.into();
        let response: ApiResponse<Value> = ApiResponse::<Value> {
            success: true,
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
    let resp = modules.todo_use_case().delete_todo(id).await;
    match resp {
        Ok(tv) => tv
            .map(|tv| {
                info!("Deleted todo `{}`.", tv.id);
                let json: JsonTodo = tv.into();
                let response: ApiResponse<Value> = ApiResponse::<Value> {
                    success: true,
                    message: "success".to_string(),
                    data: Some(json!({
                        "todoView": json,
                    })),
                };
                (StatusCode::OK, Json(response))
            })
            .ok_or_else(|| {
                error!("Todo is not found.");
                AppError::Error("data not found".to_string())
            }),
        Err(err) => {
            error!("Unexpected error: {:?}", err);
            Err(AppError::Error(err.to_string()))
        }
    }
}
