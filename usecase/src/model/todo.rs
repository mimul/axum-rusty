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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use domain::model::todo::status::TodoStatus;
    use domain::model::todo::Todo;
    use domain::model::Id;

    fn make_todo() -> (Todo, String) {
        let id: Id<domain::model::todo::Todo> = Id::gen();
        let id_str = id.value.to_string();
        let todo = Todo {
            id,
            title: "Test Todo".to_string(),
            description: "Some desc".to_string(),
            status: TodoStatus::new(Id::gen(), "OPEN".to_string(), "Open".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        (todo, id_str)
    }

    #[test]
    fn todo_view_from_todo_maps_all_fields() {
        let (todo, id_str) = make_todo();
        let view = TodoView::from(todo);
        assert_eq!(view.id, id_str.to_string());
        assert_eq!(view.title, "Test Todo");
        assert_eq!(view.description, "Some desc");
        assert_eq!(view.status.code, "OPEN");
    }

    #[test]
    fn create_todo_try_into_new_todo_generates_id() {
        let ct = CreateTodo::new("My Task".to_string(), "Details".to_string());
        let nt: NewTodo = ct.try_into().unwrap();
        assert_eq!(nt.title, "My Task");
        assert_eq!(nt.description, "Details");
    }
}
