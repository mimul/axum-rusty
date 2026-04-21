use crate::context::errors::AppError;
use fancy_regex::Regex;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use usecase::model::user::{CreateUser, LoginUser, SearchUserCondition, UserView};
use utoipa::{IntoParams, ToSchema};
use validator::{Validate, ValidationError};

static DIGIT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\d").unwrap());
static SPECIAL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^\da-zA-Z]").unwrap());
static LENGTH_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r".{7,}").unwrap());
fn validate_password(value: &str) -> Result<(), ValidationError> {
    if DIGIT_REGEX.is_match(value).unwrap()
        && SPECIAL_REGEX.is_match(value).unwrap()
        && LENGTH_REGEX.is_match(value).unwrap()
    {
        Ok(())
    } else {
        Err(ValidationError::new(""))
    }
}

#[derive(Deserialize, Debug, Validate, ToSchema, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct JsonCreateUser {
    #[validate(email(message = "invalid email"))]
    pub username: Option<String>,
    #[validate(custom(
        function = "validate_password",
        message = "password must contain one digit, one special character and must be at least 8 characters long"
    ))]
    pub password: Option<String>,
    #[validate(
        length(
            min = 2,
            max = 30,
            message = "fullname must be between 3 and 30 characters"
        ),
        required(message = "fullname is null")
    )]
    pub fullname: Option<String>,
}

impl TryFrom<JsonCreateUser> for CreateUser {
    type Error = AppError;

    fn try_from(jcu: JsonCreateUser) -> Result<Self, Self::Error> {
        Ok(CreateUser {
            username: jcu
                .username
                .ok_or_else(|| AppError::Error("`username` is required".to_string()))?,
            password: jcu
                .password
                .ok_or_else(|| AppError::Error("`password` is required".to_string()))?,
            fullname: jcu
                .fullname
                .ok_or_else(|| AppError::Error("`fullname` is required".to_string()))?,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub fullname: String,
}

impl From<UserView> for JsonUser {
    fn from(uv: UserView) -> Self {
        Self {
            id: uv.id,
            username: uv.username,
            email: uv.email,
            fullname: uv.fullname,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct UserQuery {
    pub username: String,
}

impl From<UserQuery> for SearchUserCondition {
    fn from(uq: UserQuery) -> Self {
        Self {
            username: uq.username.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Deserialize, Debug, Validate, ToSchema, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct JsonLoginUser {
    #[validate(email(message = "invalid email"))]
    pub username: Option<String>,
    #[validate(custom(
        function = "validate_password",
        message = "password must contain one digit, one special character and must be at least 8 characters long"
    ))]
    pub password: Option<String>,
}

impl TryFrom<JsonLoginUser> for LoginUser {
    type Error = AppError;

    fn try_from(jcu: JsonLoginUser) -> Result<Self, Self::Error> {
        Ok(LoginUser {
            username: jcu
                .username
                .ok_or_else(|| AppError::Error("`username` is required".to_string()))?,
            password: jcu
                .password
                .ok_or_else(|| AppError::Error("`password` is required".to_string()))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use usecase::model::user::UserView;

    #[test]
    fn validate_password_with_valid_password_returns_ok() {
        assert!(validate_password("Secret1!").is_ok());
    }

    #[test]
    fn validate_password_without_digit_returns_error() {
        assert!(validate_password("NoDigit!").is_err());
    }

    #[test]
    fn validate_password_without_special_char_returns_error() {
        assert!(validate_password("NoSpecial1").is_err());
    }

    #[test]
    fn validate_password_too_short_returns_error() {
        assert!(validate_password("Sh0rt!").is_err());
    }

    #[test]
    fn json_user_from_user_view_maps_all_fields() {
        let view = UserView {
            id: "user-id-01".to_string(),
            username: "alice@example.com".to_string(),
            email: "alice@example.com".to_string(),
            fullname: "Alice".to_string(),
        };
        let json = JsonUser::from(view);
        assert_eq!(json.id, "user-id-01");
        assert_eq!(json.username, "alice@example.com");
        assert_eq!(json.fullname, "Alice");
    }

    #[test]
    fn user_query_from_search_condition_maps_username() {
        let query = UserQuery {
            username: "bob@example.com".to_string(),
        };
        let condition: SearchUserCondition = query.into();
        assert_eq!(condition.username, Some("bob@example.com".to_string()));
    }
}
