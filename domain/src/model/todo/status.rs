use crate::model::Id;

/// Todo 상태 코드 — DB `todo_statuses.code` 컬럼의 유효값.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TodoStatusCode {
    New,
    Working,
    Waiting,
    Done,
    Discontinued,
    Pending,
    Deleted,
}

impl TodoStatusCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Working => "working",
            Self::Waiting => "waiting",
            Self::Done => "done",
            Self::Discontinued => "discontinued",
            Self::Pending => "pending",
            Self::Deleted => "deleted",
        }
    }
}

impl TryFrom<&str> for TodoStatusCode {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "new" => Ok(Self::New),
            "working" => Ok(Self::Working),
            "waiting" => Ok(Self::Waiting),
            "done" => Ok(Self::Done),
            "discontinued" => Ok(Self::Discontinued),
            "pending" => Ok(Self::Pending),
            "deleted" => Ok(Self::Deleted),
            other => Err(anyhow::anyhow!("unknown status code: {other}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TodoStatus {
    pub id: Id<TodoStatus>,
    pub code: TodoStatusCode,
    pub name: String,
}

impl TodoStatus {
    pub fn new(id: Id<TodoStatus>, code: TodoStatusCode, name: String) -> Self {
        Self { id, code, name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Id;
    use ulid::Ulid;

    #[test]
    fn todo_status_new_stores_all_fields() {
        let ulid = Ulid::new();
        let status = TodoStatus::new(Id::new(ulid), TodoStatusCode::New, "신규".to_string());
        assert_eq!(status.id.value, ulid);
        assert_eq!(status.code, TodoStatusCode::New);
        assert_eq!(status.name, "신규");
    }

    #[test]
    fn todo_status_code_try_from_valid_str() {
        assert_eq!(TodoStatusCode::try_from("new").unwrap(), TodoStatusCode::New);
        assert_eq!(
            TodoStatusCode::try_from("done").unwrap(),
            TodoStatusCode::Done
        );
    }

    #[test]
    fn todo_status_code_try_from_invalid_str_returns_error() {
        assert!(TodoStatusCode::try_from("invalid").is_err());
    }

    #[test]
    fn todo_status_code_as_str_roundtrips() {
        let codes = [
            TodoStatusCode::New,
            TodoStatusCode::Working,
            TodoStatusCode::Waiting,
            TodoStatusCode::Done,
            TodoStatusCode::Discontinued,
            TodoStatusCode::Pending,
            TodoStatusCode::Deleted,
        ];
        for code in codes {
            let s = code.as_str();
            assert_eq!(TodoStatusCode::try_from(s).unwrap(), code);
        }
    }
}
