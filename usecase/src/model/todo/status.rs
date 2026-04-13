use domain::model::todo::status::TodoStatus;
#[derive(Debug, Clone)]
pub struct TodoStatusView {
    pub id: String,
    pub code: String,
    pub name: String,
}

impl From<TodoStatus> for TodoStatusView {
    fn from(ts: TodoStatus) -> Self {
        Self {
            id: ts.id.value.to_string(),
            code: ts.code,
            name: ts.name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::model::todo::status::TodoStatus;
    use domain::model::Id;

    #[test]
    fn todo_status_view_from_todo_status_maps_all_fields() {
        let id: Id<TodoStatus> = Id::gen();
        let ulid_str = id.value.to_string();
        let status = TodoStatus::new(id, "DONE".to_string(), "Done".to_string());
        let view = TodoStatusView::from(status);
        assert_eq!(view.id, ulid_str);
        assert_eq!(view.code, "DONE");
        assert_eq!(view.name, "Done");
    }
}
