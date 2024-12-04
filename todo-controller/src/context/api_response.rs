use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::log::error;
use crate::context::errors::AppError;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<Data> {
    pub success: bool,
    pub message: String,
    pub data: Option<Data>,
}

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ApiSuccessResponse<Data> {
//     pub success: bool,
//     pub message: String,
//     pub data: Option<Data>,
// }
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            AppError::Validation(validation_errors) => {
                error!("{:?}", validation_errors);
                let mut messages: Vec<String> = Vec::new();
                let errors = validation_errors.field_errors();
                for (_, v) in errors.into_iter() {
                    for validation_error in v {
                        if let Some(msg) = validation_error.clone().message {
                            messages.push(msg.to_string());
                        }
                    }
                }
                // let error_response: ApiResponse<String> = ApiResponse::<String> {
                //     success: false,
                //     message: messages.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
                //     data: None,
                // };
                (StatusCode::BAD_REQUEST, messages.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
            }
            AppError::JsonRejection(rejection) => {
                error!("{:?}", rejection);
                // let error_response: ApiResponse<String> = ApiResponse::<String> {
                //     success: false,
                //     message: rejection.to_string(),
                //     data: None,
                // };
                (StatusCode::BAD_REQUEST, rejection.to_string())
            }
            AppError::ApiPathRejection(rejection) => {
                error!("{:?}", rejection);
                // let error_response: ApiResponse<String> = ApiResponse::<String> {
                //     success: false,
                //     message: rejection.to_string(),
                //     data: None,
                // };
                (StatusCode::BAD_REQUEST, rejection.to_string())
            }
            AppError::UnknownApiVerRejection(version) => {
                let err = format!("Unknown api version ({}).", version);
                error!("{}", err);
                // let error_response: ApiResponse<String> = ApiResponse::<String> {
                //     success: false,
                //     message: err,
                //     data: None,
                // };
                (StatusCode::BAD_REQUEST, err)
            }
            AppError::Error(error) => {
                let err = format!("error ({}).", error);
                error!("{}", err);
                // let error_response: ApiResponse<String> = ApiResponse::<String> {
                //     success: false,
                //     message: err,
                //     data: None,
                // };
                (StatusCode::OK, err)
            }
        };
        let response: ApiResponse<String> = ApiResponse::<String> {
            success: false,
            message: error_message,
            data: None,
        };

        //build up the response status code and the response content
        (status_code, Json(response)).into_response()
    }
}