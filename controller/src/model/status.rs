use serde::Serialize;
use usecase::model::todo::status::TodoStatusView;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonTodoStatus {
    pub code: String,
    pub name: String,
}

impl From<TodoStatusView> for JsonTodoStatus {
    fn from(sv: TodoStatusView) -> Self {
        Self {
            code: sv.code,
            name: sv.name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use usecase::model::todo::status::TodoStatusView;

    #[test]
    fn json_todo_status_from_todo_status_view_maps_code_and_name() {
        let view = TodoStatusView {
            id: "01JRWBKE4KE4P9MQNHCX4F0000".to_string(),
            code: "IN_PROGRESS".to_string(),
            name: "In Progress".to_string(),
        };
        let json: JsonTodoStatus = view.into();
        assert_eq!(json.code, "IN_PROGRESS");
        assert_eq!(json.name, "In Progress");
    }
}
