use crate::context::errors::AppError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<Data> {
    pub result: bool,
    pub message: String,
    pub data: Option<Data>,
}

impl<Data> ApiResponse<Data> {
    pub fn success(message: impl Into<String>, data: Data) -> Self {
        Self {
            result: true,
            message: message.into(),
            data: Some(data),
        }
    }
}

pub(crate) fn internal_error(err: impl std::fmt::Debug) -> AppError {
    error!(error = ?err, "internal error");
    AppError::Error("서버 오류가 발생했습니다".to_string())
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            AppError::InvalidJwt(_) => {
                error!("invalid or missing JWT");
                (StatusCode::UNAUTHORIZED, "인증이 필요합니다".to_string())
            }
            AppError::Forbidden(_) => {
                error!("access forbidden");
                (StatusCode::FORBIDDEN, "접근이 거부되었습니다".to_string())
            }
            AppError::Validation(validation_errors) => {
                let messages: Vec<String> = validation_errors
                    .field_errors()
                    .into_values()
                    .flat_map(|v| {
                        v.iter()
                            .filter_map(|e| e.message.as_deref().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .collect();
                error!(messages = %messages.join(", "), "validation failed");
                (StatusCode::BAD_REQUEST, messages.join(" or "))
            }
            AppError::JsonRejection(rejection) => {
                error!(error = %rejection, "JSON rejection");
                (StatusCode::BAD_REQUEST, rejection.to_string())
            }
            AppError::ApiPathRejection(rejection) => {
                error!(error = %rejection, "path rejection");
                (StatusCode::BAD_REQUEST, rejection.to_string())
            }
            AppError::UnknownApiVerRejection(version) => {
                error!(version = %version, "unknown API version");
                (
                    StatusCode::BAD_REQUEST,
                    format!("Unknown api version({version})."),
                )
            }
            AppError::Error(error) => {
                error!(error = %error, "application error");
                (StatusCode::OK, format!("error({error})."))
            }
        };
        let response: ApiResponse<String> = ApiResponse::<String> {
            result: false,
            message: error_message,
            data: None,
        };

        (status_code, Json(response)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::errors::AppError;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[test]
    fn app_error_invalid_jwt_returns_unauthorized() {
        let err = AppError::InvalidJwt("expired-token".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn app_error_forbidden_returns_forbidden_status() {
        let err = AppError::Forbidden("forbidden".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn app_error_unknown_ver_rejection_returns_bad_request() {
        let err = AppError::UnknownApiVerRejection("v99".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn app_error_error_variant_returns_ok_status() {
        let err = AppError::Error("something went wrong".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn api_response_fields_set_correctly() {
        let resp: ApiResponse<String> = ApiResponse {
            result: true,
            message: "success".to_string(),
            data: Some("payload".to_string()),
        };
        assert!(resp.result);
        assert_eq!(resp.message, "success");
        assert_eq!(resp.data, Some("payload".to_string()));
    }

    #[test]
    fn api_response_with_none_data_has_no_payload() {
        let resp: ApiResponse<String> = ApiResponse {
            result: false,
            message: "error".to_string(),
            data: None,
        };
        assert!(!resp.result);
        assert!(resp.data.is_none());
    }

    #[test]
    fn api_response_serializes_to_json() {
        let resp: ApiResponse<String> = ApiResponse {
            result: true,
            message: "ok".to_string(),
            data: Some("value".to_string()),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["result"], true);
        assert_eq!(json["message"], "ok");
        assert_eq!(json["data"], "value");
    }

    #[test]
    fn api_response_null_data_serializes_as_null() {
        let resp: ApiResponse<String> = ApiResponse {
            result: false,
            message: "err".to_string(),
            data: None,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json["data"].is_null());
    }
}
