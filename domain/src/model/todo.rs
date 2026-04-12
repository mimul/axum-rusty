pub mod status;

use crate::model::todo::status::TodoStatus;
use crate::model::Id;
use chrono::{DateTime, Utc};

pub struct Todo {
    pub id: Id<Todo>,
    pub title: String,
    pub description: String,
    pub status: TodoStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewTodo {
    pub id: Id<Todo>,
    pub title: String,
    pub description: String,
}

impl NewTodo {
    pub fn new(id: Id<Todo>, title: String, description: String) -> Self {
        Self {
            id,
            title,
            description,
        }
    }
}

pub struct UpdateTodo {
    pub id: Id<Todo>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TodoStatus>,
}

impl UpdateTodo {
    pub fn new(
        id: Id<Todo>,
        title: Option<String>,
        description: Option<String>,
        status: Option<TodoStatus>,
    ) -> Self {
        Self {
            id,
            title,
            description,
            status,
        }
    }
}

pub struct UpsertTodo {
    pub id: Id<Todo>,
    pub title: String,
    pub description: String,
    pub status: TodoStatus,
}

impl UpsertTodo {
    pub fn new(id: Id<Todo>, title: String, description: String, status: TodoStatus) -> Self {
        Self {
            id,
            title,
            description,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Id;
    use ulid::Ulid;

    fn make_status() -> TodoStatus {
        TodoStatus::new(Id::new(Ulid::new()), "OPEN".to_string(), "Open".to_string())
    }

    #[test]
    fn new_todo_new_stores_all_fields() {
        let ulid = Ulid::new();
        let nt = NewTodo::new(Id::new(ulid), "Title".to_string(), "Desc".to_string());
        assert_eq!(nt.id.value, ulid);
        assert_eq!(nt.title, "Title");
        assert_eq!(nt.description, "Desc");
    }

    #[test]
    fn update_todo_new_stores_optional_fields() {
        let ulid = Ulid::new();
        let status = make_status();
        let ut = UpdateTodo::new(
            Id::new(ulid),
            Some("New Title".to_string()),
            None,
            Some(status),
        );
        assert_eq!(ut.id.value, ulid);
        assert_eq!(ut.title, Some("New Title".to_string()));
        assert!(ut.description.is_none());
        assert!(ut.status.is_some());
    }

    #[test]
    fn upsert_todo_new_stores_all_fields() {
        let ulid = Ulid::new();
        let status = make_status();
        let status_ulid = status.id.value;
        let ut = UpsertTodo::new(
            Id::new(ulid),
            "Title".to_string(),
            "Desc".to_string(),
            status,
        );
        assert_eq!(ut.id.value, ulid);
        assert_eq!(ut.title, "Title");
        assert_eq!(ut.description, "Desc");
        assert_eq!(ut.status.id.value, status_ulid);
    }
}
