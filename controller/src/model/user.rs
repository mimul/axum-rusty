use fancy_regex::Regex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use usecase::model::user::{CreateUser, LoginUser, SearchUserCondition, UserView};
use utoipa::{IntoParams, ToSchema};
use validator::{Validate, ValidationError};

lazy_static! {
    //static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,4}$").unwrap();
    static ref DIGIT_REGEX: Regex = Regex::new(r"\d").unwrap();
    static ref SPECIAL_REGEX: Regex = Regex::new(r"[^\da-zA-Z]").unwrap();
    static ref LENGTH_REGEX: Regex = Regex::new(r".{7,}").unwrap();
}

fn validate_password(value: &str) -> Result<(), ValidationError> {
    if DIGIT_REGEX.is_match(value).unwrap() && SPECIAL_REGEX.is_match(value).unwrap() && LENGTH_REGEX.is_match(value).unwrap()
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
    #[validate(length(
        min = 2,
        max = 30,
        message = "fullname must be between 3 and 30 characters"
    ), required(message = "fullname is null"))]
    pub fullname: Option<String>,
}

impl From<JsonCreateUser> for CreateUser {
    fn from(jcu: JsonCreateUser) -> Self {
        CreateUser {
            username: jcu.username.unwrap(),
            password: jcu.password.unwrap(),
            fullname: jcu.fullname.unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub fullname: String,
}

impl From<UserView> for JsonUser {
    fn from(uv: UserView) -> Self {
        Self {
            id: uv.id,
            username: uv.username,
            email: uv.email,
            password: uv.password,
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
    pub exp: usize,
    pub sub: String,
    pub username: String,
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

impl From<JsonLoginUser> for LoginUser {
    fn from(jcu: JsonLoginUser) -> Self {
        LoginUser {
            username: jcu.username.unwrap(),
            password: jcu.password.unwrap(),
        }
    }
}