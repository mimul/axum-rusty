use domain::model::todo::status::TodoStatus;
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
            code: ts.code,
            name: ts.name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::model::Id;

    #[test]
    fn stored_todo_status_try_into_todo_status_succeeds_with_valid_id() {
        let id: Id<domain::model::todo::status::TodoStatus> = Id::gen();
        let ulid = id.value;
        let stored = StoredTodoStatus {
            id: ulid.to_string(),
            code: "OPEN".to_string(),
            name: "Open".to_string(),
        };
        let status: domain::model::todo::status::TodoStatus = stored.try_into().unwrap();
        assert_eq!(status.id.value, ulid);
        assert_eq!(status.code, "OPEN");
        assert_eq!(status.name, "Open");
    }

    #[test]
    fn stored_todo_status_try_into_todo_status_fails_with_invalid_id() {
        let stored = StoredTodoStatus {
            id: "not-a-ulid".to_string(),
            code: "OPEN".to_string(),
            name: "Open".to_string(),
        };
        let result: Result<domain::model::todo::status::TodoStatus, _> = stored.try_into();
        assert!(result.is_err());
    }
}
