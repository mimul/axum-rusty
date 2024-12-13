use serde::{Deserialize, Serialize};
use todo_usecase::model::user::{CreateUser, SearchUserCondition, UserView};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Deserialize, Debug, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JsonCreateUser {
    #[validate(
        length(min = 5, message = "`username` is empty."),
        required(message = "`username` is null.")
    )]
    pub username: Option<String>,
    #[validate(required(message = "`password` is null."))]
    pub password: Option<String>,
}

impl From<JsonCreateUser> for CreateUser {
    fn from(jcu: JsonCreateUser) -> Self {
        CreateUser {
            username: jcu.username.unwrap(),
            password: jcu.password.unwrap(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

impl From<UserView> for JsonUser {
    fn from(uv: UserView) -> Self {
        Self {
            id: uv.id,
            username: uv.username,
            email: uv.email,
            password: uv.password,
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
