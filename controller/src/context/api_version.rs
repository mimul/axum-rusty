use crate::context::errors::AppError;
use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;
use axum::{async_trait, RequestPartsExt};
use serde::Deserialize;
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum ApiVersion {
    V1,
}

#[async_trait]
impl<S> FromRequestParts<S> for ApiVersion
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let params: Path<HashMap<String, String>> = parts.extract().await?;

        let version = params
            .get("v")
            .ok_or_else(|| AppError::UnknownApiVerRejection("missing version param".to_string()))?;

        match version.as_str() {
            "v1" => Ok(ApiVersion::V1),
            _ => Err(AppError::UnknownApiVerRejection(version.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_version_v1_deserializes_from_string() {
        // rename_all = "camelCase": V1 → "v1"
        let v: ApiVersion = serde_json::from_str(r#""v1""#).unwrap();
        assert!(matches!(v, ApiVersion::V1));
    }

    #[test]
    fn api_version_unknown_fails_deserialization() {
        let result: Result<ApiVersion, _> = serde_json::from_str(r#""v99""#);
        assert!(result.is_err());
    }
}
