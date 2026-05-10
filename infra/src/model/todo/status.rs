use domain::model::todo::status::{TodoStatus, TodoStatusCode};
use sqlx::FromRow;

#[derive(FromRow, Debug)]
pub struct StoredTodoStatus {
    pub id: String,
    pub code: String,
    pub name: String,
}

impl TryFrom<StoredTodoStatus> for TodoStatus {
    type Error = anyhow::Error;

    fn try_from(ts: StoredTodoStatus) -> Result<Self, Self::Error> {
        Ok(TodoStatus {
            id: ts.id.try_into()?,
            code: TodoStatusCode::try_from(ts.code.as_str())?,
            name: ts.name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::model::Id;

    #[test]
    fn stored_todo_status_try_into_todo_status_succeeds_with_valid_code() {
        let id: Id<domain::model::todo::status::TodoStatus> = Id::gen();
        let ulid = id.value;
        let stored = StoredTodoStatus {
            id: ulid.to_string(),
            code: "new".to_string(),
            name: "신규".to_string(),
        };
        let status: domain::model::todo::status::TodoStatus = stored.try_into().unwrap();
        assert_eq!(status.id.value, ulid);
        assert_eq!(status.code, TodoStatusCode::New);
        assert_eq!(status.name, "신규");
    }

    #[test]
    fn stored_todo_status_try_into_todo_status_fails_with_invalid_id() {
        let stored = StoredTodoStatus {
            id: "not-a-ulid".to_string(),
            code: "new".to_string(),
            name: "신규".to_string(),
        };
        let result: Result<domain::model::todo::status::TodoStatus, _> = stored.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn stored_todo_status_try_into_todo_status_fails_with_unknown_code() {
        let id: Id<domain::model::todo::status::TodoStatus> = Id::gen();
        let stored = StoredTodoStatus {
            id: id.value.to_string(),
            code: "unknown_status".to_string(),
            name: "Unknown".to_string(),
        };
        let result: Result<domain::model::todo::status::TodoStatus, _> = stored.try_into();
        assert!(result.is_err());
    }
}
