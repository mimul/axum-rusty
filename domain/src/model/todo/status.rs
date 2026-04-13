use crate::model::Id;

#[derive(Debug)]
pub struct TodoStatus {
    pub id: Id<TodoStatus>,
    pub code: String,
    pub name: String,
}

impl TodoStatus {
    pub fn new(id: Id<TodoStatus>, code: String, name: String) -> Self {
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
        let status = TodoStatus::new(Id::new(ulid), "OPEN".to_string(), "Open".to_string());
        assert_eq!(status.id.value, ulid);
        assert_eq!(status.code, "OPEN");
        assert_eq!(status.name, "Open");
    }
}
