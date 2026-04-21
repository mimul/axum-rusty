pub mod status;

use chrono::{DateTime, Utc};
use domain::model::todo::status::TodoStatus;
use domain::model::todo::{NewTodo, Todo, UpdateTodo, UpsertTodo};
use sqlx::FromRow;

#[derive(FromRow, Debug)]
pub struct StoredTodo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status_id: String,
    pub status_code: String,
    pub status_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<StoredTodo> for Todo {
    type Error = anyhow::Error;

    fn try_from(t: StoredTodo) -> Result<Self, Self::Error> {
        Ok(Todo {
            id: t.id.try_into()?,
            title: t.title,
            description: t.description,
            status: TodoStatus::new(t.status_id.try_into()?, t.status_code, t.status_name),
            created_at: t.created_at,
            updated_at: t.updated_at,
        })
    }
}

#[derive(FromRow, Debug)]
pub struct InsertTodo {
    pub id: String,
    pub title: String,
    pub description: String,
}

impl From<NewTodo> for InsertTodo {
    fn from(nt: NewTodo) -> Self {
        InsertTodo {
            id: nt.id.value.to_string(),
            title: nt.title,
            description: nt.description,
        }
    }
}

pub struct UpdateStoredTodo {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status_id: Option<String>,
}

impl From<UpdateTodo> for UpdateStoredTodo {
    fn from(ut: UpdateTodo) -> Self {
        let status_id = ut.status.map(|s| s.id.value.to_string());

        UpdateStoredTodo {
            id: ut.id.value.to_string(),
            title: ut.title,
            description: ut.description,
            status_id,
        }
    }
}

pub struct UpsertStoredTodo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status_id: String,
}

impl From<UpsertTodo> for UpsertStoredTodo {
    fn from(ut: UpsertTodo) -> Self {
        UpsertStoredTodo {
            id: ut.id.value.to_string(),
            title: ut.title,
            description: ut.description,
            status_id: ut.status.id.value.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use domain::model::todo::status::TodoStatus;
    use domain::model::todo::{NewTodo, Todo, UpdateTodo, UpsertTodo};
    use domain::model::Id;

    fn make_todo_status() -> TodoStatus {
        TodoStatus::new(Id::gen(), "OPEN".to_string(), "Open".to_string())
    }

    #[test]
    fn insert_todo_from_new_todo_maps_all_fields() {
        let id: Id<domain::model::todo::Todo> = Id::gen();
        let ulid_str = id.value.to_string();
        let nt = NewTodo::new(id, "Task".to_string(), "Do it".to_string());
        let insert: InsertTodo = nt.into();
        assert_eq!(insert.id, ulid_str);
        assert_eq!(insert.title, "Task");
        assert_eq!(insert.description, "Do it");
    }

    #[test]
    fn stored_todo_try_into_todo_succeeds_with_valid_ids() {
        let todo_id: Id<domain::model::todo::Todo> = Id::gen();
        let status_id: Id<domain::model::todo::status::TodoStatus> = Id::gen();
        let todo_ulid = todo_id.value;
        let status_ulid = status_id.value;
        let now = Utc::now();
        let stored = StoredTodo {
            id: todo_ulid.to_string(),
            title: "My Todo".to_string(),
            description: "Details".to_string(),
            status_id: status_ulid.to_string(),
            status_code: "OPEN".to_string(),
            status_name: "Open".to_string(),
            created_at: now,
            updated_at: now,
        };
        let todo: Todo = stored.try_into().unwrap();
        assert_eq!(todo.id.value, todo_ulid);
        assert_eq!(todo.title, "My Todo");
        assert_eq!(todo.status.id.value, status_ulid);
    }

    #[test]
    fn update_stored_todo_from_update_todo_with_status() {
        let id: Id<domain::model::todo::Todo> = Id::gen();
        let ulid_str = id.value.to_string();
        let status = make_todo_status();
        let status_ulid = status.id.value;
        let ut = UpdateTodo::new(id, Some("Updated".to_string()), None, Some(status));
        let stored: UpdateStoredTodo = ut.into();
        assert_eq!(stored.id, ulid_str);
        assert_eq!(stored.title, Some("Updated".to_string()));
        assert!(stored.description.is_none());
        assert_eq!(stored.status_id, Some(status_ulid.to_string()));
    }

    #[test]
    fn upsert_stored_todo_from_upsert_todo_maps_all_fields() {
        let id: Id<domain::model::todo::Todo> = Id::gen();
        let ulid_str = id.value.to_string();
        let status = make_todo_status();
        let status_ulid = status.id.value;
        let ut = UpsertTodo::new(id, "Title".to_string(), "Desc".to_string(), status);
        let stored: UpsertStoredTodo = ut.into();
        assert_eq!(stored.id, ulid_str);
        assert_eq!(stored.title, "Title");
        assert_eq!(stored.description, "Desc");
        assert_eq!(stored.status_id, status_ulid.to_string());
    }
}
