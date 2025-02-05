pub mod status;

use crate::model::todo::{InsertTodo, StoredTodo, UpdateStoredTodo, UpsertStoredTodo};
use crate::repository::DatabaseRepositoryImpl;
use async_trait::async_trait;
use domain::model::todo::status::TodoStatus;
use domain::model::todo::{NewTodo, Todo, UpdateTodo, UpsertTodo};
use domain::model::Id;
use domain::repository::todo::TodoRepository;
use sqlx::{query, query_as};
use domain::transaction::PostgresAcquire;

#[async_trait]
impl TodoRepository for DatabaseRepositoryImpl<Todo> {
    async fn get(&self, id: &Id<Todo>, executor: impl PostgresAcquire<'_>) -> anyhow::Result<Option<Todo>> {
        let mut conn = executor.acquire().await?;
        let sql = r#"
            select t.id as id, t.title as title, t.description as description, ts.id as status_id, ts.code as status_code, ts.name as status_name,
                t.created_at as created_at, t.updated_at as updated_at
            from  todos as t
            inner join todo_statuses as ts on ts.id = t.status_id
            where t.id = $1
        "#;
        let stored_todo = query_as::<_, StoredTodo>(sql)
            .bind(id.value.to_string())
            .fetch_one(&mut *conn)
            .await
            .ok();

        match stored_todo {
            Some(st) => Ok(Some(st.try_into()?)),
            None => Ok(None),
        }
    }

    async fn find(&self, status: Option<TodoStatus>, executor: impl PostgresAcquire<'_>) -> anyhow::Result<Option<Vec<Todo>>> {
        let mut conn = executor.acquire().await?;
        let where_status = if let Some(s) = &status {
            s.id.value.to_string()
        } else {
            "".to_string()
        };

        let mut sql = r#"
            select t.id as id, t.title as title, t.description as description, ts.id as status_id, ts.code as status_code, ts.name as status_name,
            t.created_at as created_at, t.updated_at as updated_at
            from  todos as t
            inner join todo_statuses as ts on ts.id = t.status_id
            where t.status_id in ($1)
            order by t.created_at asc
        "#
        .to_string();

        if status.is_none() {
            sql = sql.replace("$1", "select id from todo_statuses");
        }

        let stored_todo_list = query_as::<_, StoredTodo>(&sql)
            .bind(where_status)
            .fetch_all(&mut *conn)
            .await
            .ok();

        match stored_todo_list {
            Some(todo_list) => {
                let todos = todo_list.into_iter().flat_map(|st| st.try_into()).collect();
                Ok(Some(todos))
            }
            None => Ok(None),
        }
    }

    async fn insert(&self, source: NewTodo, executor: impl PostgresAcquire<'_>) -> anyhow::Result<Todo> {
        let mut conn = executor.acquire().await?;
        let todo: InsertTodo = source.into();
        let id = todo.id.clone();

        let _ = query("insert into todos (id, title, description) values ($1, $2, $3)")
            .bind(todo.id)
            .bind(todo.title)
            .bind(todo.description)
            .execute(&mut *conn)
            .await?;

        let sql = r#"
            select t.id as id, t.title as title, t.description as description, ts.id as status_id, ts.code as status_code, ts.name as status_name,
            t.created_at as created_at, t.updated_at as updated_at
            from  todos as t
            inner join todo_statuses as ts on ts.id = t.status_id
            where t.id = $1
        "#;

        let stored_todo = query_as::<_, StoredTodo>(sql)
            .bind(id)
            .fetch_one(&mut *conn)
            .await?;
        Ok(stored_todo.try_into()?)
    }

    async fn update(&self, source: UpdateTodo, executor: impl PostgresAcquire<'_>) -> anyhow::Result<Todo> {
        let mut conn = executor.acquire().await?;
        let todo: UpdateStoredTodo = source.into();
        let id = todo.id.clone();

        let update_sql = r#"
            update todos as target set
                title = case when $2 is not null then $2 else current_todo.title end,
                description = case when $3 is not null then $3 else current_todo.description end,
                status_id = case when $4 is not null then $4 else current_todo.status_id end,
                updated_at = current_timestamp
            from  (select * from todos where id = $1) as current_todo
            where target.id = $1
        "#;

        let _ = query(update_sql)
            .bind(todo.id)
            .bind(todo.title)
            .bind(todo.description)
            .bind(todo.status_id)
            .execute(&mut *conn)
            .await?;

        let sql = r#"
            select t.id as id, t.title as title, t.description as description, ts.id as status_id, ts.code as status_code, ts.name as status_name,
                t.created_at as created_at, t.updated_at as updated_at
            from todos as t
            inner join todo_statuses as ts on ts.id = t.status_id
            where t.id = $1
        "#;

        let stored_todo = query_as::<_, StoredTodo>(sql)
            .bind(id)
            .fetch_one(&mut *conn)
            .await?;
        Ok(stored_todo.try_into()?)
    }

    async fn upsert(&self, source: UpsertTodo, executor: impl PostgresAcquire<'_>) -> anyhow::Result<Todo> {
        let mut conn = executor.acquire().await?;
        let todo: UpsertStoredTodo = source.into();
        let id = todo.id.clone();

        let upsert_sql = r#"
            insert into todos (id, title, description, status_id) values ($1, $2, $3, $4)
            on conflict on constraint pk_todos_id
            do update set title = $2, description = $3, status_id = $4, updated_at = current_timestamp
        "#;

        let _ = query(upsert_sql)
            .bind(todo.id)
            .bind(todo.title)
            .bind(todo.description)
            .bind(todo.status_id)
            .execute(&mut *conn)
            .await?;

        let sql = r#"
            select t.id as id, t.title as title, t.description as description, ts.id as status_id, ts.code as status_code, ts.name as status_name,
                t.created_at as created_at, t.updated_at as updated_at
            from  todos as t
            inner join todo_statuses as ts on ts.id = t.status_id
            where t.id = $1
        "#;

        let stored_todo = query_as::<_, StoredTodo>(sql)
            .bind(id)
            .fetch_one(&mut *conn)
            .await?;
        Ok(stored_todo.try_into()?)
    }

    async fn delete(&self, id: &Id<Todo>, executor: impl PostgresAcquire<'_>) -> anyhow::Result<Option<Todo>> {
        let mut conn = executor.acquire().await?;

        let sql = r#"
            select t.id as id, t.title as title, t.description as description, ts.id as status_id, ts.code as status_code, ts.name as status_name,
                t.created_at as created_at, t.updated_at as updated_at
            from  todos as t
            inner join todo_statuses as ts on ts.id = t.status_id
            where t.id = $1
        "#;

        let stored_todo = query_as::<_, StoredTodo>(sql)
            .bind(id.value.to_string())
            .fetch_one(&mut *conn)
            .await
            .ok();

        match stored_todo {
            Some(st) => {
                let delete_sql = r#"
                    delete from todos where id = $1
                "#;

                let _ = query(delete_sql)
                    .bind(id.value.to_string())
                    .execute(&mut *conn)
                    .await?;

                Ok(Some(st.try_into()?))
            }
            None => Ok(None),
        }
    }
}

// #[cfg(test)]
// mod test {
//     use domain::model::todo::NewTodo;
//     use domain::model::Id;
//     use domain::repository::todo::TodoRepository;
//     use ulid::Ulid;
//     use crate::persistence::config::Config;
//     use super::DatabaseRepositoryImpl;
//     use crate::persistence::postgres::Db;
//
//     #[ignore]
//     #[tokio::test]
//     async fn test_insert_todo() {
//         let db = Db::new(Config::init()).await;
//         let repository = DatabaseRepositoryImpl::new();
//         db.clone().0.acquire().await.unwrap();
//         let id = Ulid::new();
//         let _ = repository
//             .insert(NewTodo::new(
//                 Id::new(id),
//                 "재미있는 일".to_string(),
//                 "RUST 공부 및 아키텍처 연구좀 하자.".to_string(),
//             ))
//             .await
//             .unwrap();
//         let todo = repository.get(&Id::new(id)).await.unwrap().unwrap();
//         assert_eq!(todo.id.value, id);
//     }
// }
