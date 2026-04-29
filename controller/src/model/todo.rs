use crate::context::errors::AppError;
use crate::model::status::JsonTodoStatus;
use serde::{Deserialize, Serialize};
use usecase::model::todo::{
    CreateTodo, SearchTodoCondition, TodoView, UpdateTodoView, UpsertTodoView,
};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonTodo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: JsonTodoStatus,
    pub created_at: String,
    pub updated_at: String,
}

impl From<TodoView> for JsonTodo {
    fn from(tv: TodoView) -> Self {
        Self {
            id: tv.id,
            title: tv.title,
            description: tv.description,
            status: tv.status.into(),
            created_at: tv.created_at.to_string(),
            updated_at: tv.updated_at.to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonTodoList {
    pub todos: Vec<JsonTodo>,
}

impl JsonTodoList {
    pub fn new(todos: Vec<JsonTodo>) -> Self {
        Self { todos }
    }
}

#[derive(Deserialize, Debug, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JsonCreateTodo {
    #[validate(
        length(min = 1, message = "`title` is empty."),
        required(message = "`title` is null.")
    )]
    pub title: Option<String>,
    #[validate(required(message = "`description` is null."))]
    pub description: Option<String>,
}

impl TryFrom<JsonCreateTodo> for CreateTodo {
    type Error = AppError;

    fn try_from(jc: JsonCreateTodo) -> Result<Self, Self::Error> {
        Ok(CreateTodo {
            title: jc
                .title
                .ok_or_else(|| AppError::Error("`title` is required".to_string()))?,
            description: jc
                .description
                .ok_or_else(|| AppError::Error("`description` is required".to_string()))?,
        })
    }
}

#[derive(Deserialize, Debug, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JsonUpdateTodoContents {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status_code: Option<String>,
}

impl JsonUpdateTodoContents {
    pub fn validate(self, id: String) -> Result<UpdateTodoView, Vec<String>> {
        let mut errors: Vec<String> = vec![];

        if let Some(title) = &self.title {
            if title.is_empty() {
                errors.push("`title` is empty.".to_string());
            }
        }

        if let Some(status_code) = &self.status_code {
            if status_code.is_empty() {
                errors.push("`statusCode` is empty.".to_string());
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(UpdateTodoView::new(
            id,
            self.title,
            self.description,
            self.status_code,
        ))
    }
}

#[derive(Deserialize, Debug, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JsonUpsertTodoContents {
    #[validate(
        length(min = 1, message = "`title` is empty."),
        required(message = "`title` is null.")
    )]
    pub title: Option<String>,
    #[validate(required(message = "`description` is null."))]
    pub description: Option<String>,
    #[validate(
        length(min = 1, message = "`statusCode` is empty."),
        required(message = "`statusCode` is null.")
    )]
    pub status_code: Option<String>,
}

impl JsonUpsertTodoContents {
    pub fn try_to_view(self, id: String) -> Result<UpsertTodoView, AppError> {
        Ok(UpsertTodoView::new(
            id,
            self.title
                .ok_or_else(|| AppError::Error("`title` is required".to_string()))?,
            self.description
                .ok_or_else(|| AppError::Error("`description` is required".to_string()))?,
            self.status_code
                .ok_or_else(|| AppError::Error("`statusCode` is required".to_string()))?,
        ))
    }
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct TodoQuery {
    pub status: Option<String>,
}

impl From<TodoQuery> for SearchTodoCondition {
    fn from(tq: TodoQuery) -> Self {
        Self {
            status_code: tq.status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_update_todo_validate_with_all_fields_returns_update_view() {
        let contents = JsonUpdateTodoContents {
            title: Some("New Title".to_string()),
            description: Some("New Desc".to_string()),
            status_code: Some("DONE".to_string()),
        };
        let result = contents.validate("abc123".to_string());
        assert!(result.is_ok());
        let view = result.unwrap();
        assert_eq!(view.id, "abc123");
        assert_eq!(view.title, Some("New Title".to_string()));
        assert_eq!(view.status_code, Some("DONE".to_string()));
    }

    #[test]
    fn json_update_todo_validate_with_empty_title_returns_error() {
        let contents = JsonUpdateTodoContents {
            title: Some("".to_string()),
            description: None,
            status_code: None,
        };
        let result = contents.validate("id1".to_string());
        assert!(result.is_err());
        let errs = result.err().unwrap();
        assert!(errs.iter().any(|e| e.contains("title")));
    }

    #[test]
    fn json_update_todo_validate_with_empty_status_code_returns_error() {
        let contents = JsonUpdateTodoContents {
            title: None,
            description: None,
            status_code: Some("".to_string()),
        };
        let result = contents.validate("id2".to_string());
        assert!(result.is_err());
        let errs = result.err().unwrap();
        assert!(errs.iter().any(|e| e.contains("statusCode")));
    }

    #[test]
    fn json_update_todo_validate_with_both_empty_returns_two_errors() {
        let contents = JsonUpdateTodoContents {
            title: Some("".to_string()),
            description: None,
            status_code: Some("".to_string()),
        };
        let result = contents.validate("id3".to_string());
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().len(), 2);
    }

    #[test]
    fn json_update_todo_validate_with_no_fields_returns_ok() {
        let contents = JsonUpdateTodoContents {
            title: None,
            description: None,
            status_code: None,
        };
        let result = contents.validate("id4".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn todo_query_from_search_condition_with_status_maps_correctly() {
        let query = TodoQuery {
            status: Some("OPEN".to_string()),
        };
        let condition: SearchTodoCondition = query.into();
        assert_eq!(condition.status_code, Some("OPEN".to_string()));
    }

    #[test]
    fn todo_query_from_search_condition_without_status_maps_none() {
        let query = TodoQuery { status: None };
        let condition: SearchTodoCondition = query.into();
        assert!(condition.status_code.is_none());
    }

    #[test]
    fn json_todo_list_new_with_empty_vec_stores_empty_todos() {
        let list = JsonTodoList::new(vec![]);
        assert!(list.todos.is_empty());
    }
}
