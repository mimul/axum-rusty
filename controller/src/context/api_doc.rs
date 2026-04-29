use crate::model::todo::{
    JsonCreateTodo, JsonUpdateTodoContents, JsonUpsertTodoContents, TodoQuery,
};
use crate::model::user::{JsonCreateUser, UserQuery};
use crate::routes::{todo, user};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
#[derive(OpenApi)]
#[openapi(
    paths(
        todo::get_todo, todo::find_todo, todo::create_todo, todo::update_todo, todo::upsert_todo, todo::delete_todo,
        user::create_user, user::get_user, user::get_user_by_username, user::login_user
    ),
    components(schemas(
        JsonCreateTodo, TodoQuery, JsonUpdateTodoContents, JsonUpsertTodoContents,
        JsonCreateUser, UserQuery
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Todo", description = "Todo API")
    )
)]
pub struct ApiDoc;

pub struct SecurityAddon;
impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.components.as_mut().unwrap().add_security_scheme(
            "Authorization",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}
