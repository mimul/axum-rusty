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
