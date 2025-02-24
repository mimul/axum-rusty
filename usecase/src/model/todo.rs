pub mod status;

use crate::model::todo::status::TodoStatusView;
use crate::model::DateTimeRfc3339;
use domain::model::todo::{NewTodo, Todo};
use domain::model::Id;

#[derive(Debug, Clone)]
pub struct TodoView {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TodoStatusView,
    pub created_at: DateTimeRfc3339,
    pub updated_at: DateTimeRfc3339,
}

impl From<Todo> for TodoView {
    fn from(todo: Todo) -> Self {
        Self {
            id: todo.id.value.to_string(),
            title: todo.title,
            description: todo.description,
            status: todo.status.into(),
            created_at: todo.created_at.into(),
            updated_at: todo.updated_at.into(),
        }
    }
}

pub struct CreateTodo {
    pub title: String,
    pub description: String,
}

impl CreateTodo {
    pub fn new(title: String, description: String) -> Self {
        Self { title, description }
    }
}

impl TryFrom<CreateTodo> for NewTodo {
    type Error = anyhow::Error;

    fn try_from(ct: CreateTodo) -> Result<Self, Self::Error> {
        Ok(NewTodo::new(Id::gen(), ct.title, ct.description))
    }
}

pub struct UpdateTodoView {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status_code: Option<String>,
}

impl UpdateTodoView {
    pub fn new(
        id: String,
        title: Option<String>,
        description: Option<String>,
        status_code: Option<String>,
    ) -> Self {
        Self {
            id,
            title,
            description,
            status_code,
        }
    }
}

pub struct UpsertTodoView {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status_code: String,
}

impl UpsertTodoView {
    pub fn new(id: String, title: String, description: String, status_code: String) -> Self {
        Self {
            id,
            title,
            description,
            status_code,
        }
    }
}

pub struct SearchTodoCondition {
    pub status_code: Option<String>,
}
