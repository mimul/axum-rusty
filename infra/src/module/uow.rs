use crate::repository::todo::status::PgTodoStatusRepo;
use crate::repository::todo::PgTodoRepo;
use crate::repository::user::PgUserRepo;
use anyhow::Context;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;
use usecase::module::uow::{UnitOfWork, UnitOfWorkFactory};

/// 단일 PostgreSQL 트랜잭션을 여러 리포지토리가 공유하는 타입 별칭.
///
/// `Option`으로 감싸서 commit/rollback 이후 이중 실행 방지.
pub(crate) type SharedTx = Arc<Mutex<Option<Transaction<'static, Postgres>>>>;

/// PostgreSQL 트랜잭션 컨텍스트 구현체.
pub struct PgUnitOfWork {
    shared_tx: SharedTx,
    todo_repo: PgTodoRepo,
    todo_status_repo: PgTodoStatusRepo,
    user_repo: PgUserRepo,
}

impl PgUnitOfWork {
    pub fn new(tx: Transaction<'static, Postgres>) -> Self {
        let shared_tx: SharedTx = Arc::new(Mutex::new(Some(tx)));
        Self {
            todo_repo: PgTodoRepo::new(shared_tx.clone()),
            todo_status_repo: PgTodoStatusRepo::new(shared_tx.clone()),
            user_repo: PgUserRepo::new(shared_tx.clone()),
            shared_tx,
        }
    }
}

#[async_trait]
impl UnitOfWork for PgUnitOfWork {
    fn todo_repo(&self) -> &dyn domain::repository::todo::TodoRepository {
        &self.todo_repo
    }
    fn todo_status_repo(&self) -> &dyn domain::repository::todo::status::TodoStatusRepository {
        &self.todo_status_repo
    }
    fn user_repo(&self) -> &dyn domain::repository::user::UserRepository {
        &self.user_repo
    }

    async fn commit(&mut self) -> anyhow::Result<()> {
        let tx = self
            .shared_tx
            .lock()
            .await
            .take()
            .context("transaction already committed or rolled back")?;
        tx.commit().await.map_err(Into::into)
    }

    async fn rollback(&mut self) -> anyhow::Result<()> {
        let tx = self
            .shared_tx
            .lock()
            .await
            .take()
            .context("transaction already committed or rolled back")?;
        tx.rollback().await.map_err(Into::into)
    }
}

/// `PgUnitOfWork`를 생성하는 팩토리.
pub struct PgUnitOfWorkFactory {
    pool: PgPool,
}

impl PgUnitOfWorkFactory {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UnitOfWorkFactory for PgUnitOfWorkFactory {
    async fn begin(&self) -> anyhow::Result<Box<dyn UnitOfWork>> {
        let tx = self.pool.begin().await?;
        Ok(Box::new(PgUnitOfWork::new(tx)))
    }
}
