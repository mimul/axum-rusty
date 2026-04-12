use crate::repository::todo::status::PgTodoStatusRepo;
use crate::repository::todo::PgTodoRepo;
use crate::repository::user::PgUserRepo;
use anyhow::Context;
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;
use usecase::module::uow::{
    TodoUnitOfWork, TodoUnitOfWorkFactory, UserUnitOfWork, UserUnitOfWorkFactory,
};

/// 단일 PostgreSQL 트랜잭션을 여러 레포지토리가 공유하는 타입 별칭.
///
/// `Option`으로 감싸서 commit/rollback 이후 이중 실행 방지.
pub(crate) type SharedTx = Arc<Mutex<Option<Transaction<'static, Postgres>>>>;

// ---------------------------------------------------------------------------
// Todo 도메인
// ---------------------------------------------------------------------------

/// Todo 도메인 전용 트랜잭션 컨텍스트.
pub struct PgTodoUnitOfWork {
    shared_tx: SharedTx,
    todo_repo: PgTodoRepo,
    todo_status_repo: PgTodoStatusRepo,
}

impl PgTodoUnitOfWork {
    fn new(tx: Transaction<'static, Postgres>) -> Self {
        let shared_tx: SharedTx = Arc::new(Mutex::new(Some(tx)));
        Self {
            todo_repo: PgTodoRepo::new(shared_tx.clone()),
            todo_status_repo: PgTodoStatusRepo::new(shared_tx.clone()),
            shared_tx,
        }
    }
}

#[async_trait]
impl TodoUnitOfWork for PgTodoUnitOfWork {
    fn todo_repo(&self) -> &dyn domain::repository::todo::TodoRepository {
        &self.todo_repo
    }
    fn todo_status_repo(&self) -> &dyn domain::repository::todo::status::TodoStatusRepository {
        &self.todo_status_repo
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

/// Todo 도메인 UoW 팩토리.
pub struct PgTodoUnitOfWorkFactory {
    pool: PgPool,
}

impl PgTodoUnitOfWorkFactory {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TodoUnitOfWorkFactory for PgTodoUnitOfWorkFactory {
    async fn begin(&self) -> anyhow::Result<Box<dyn TodoUnitOfWork>> {
        let tx = self.pool.begin().await?;
        Ok(Box::new(PgTodoUnitOfWork::new(tx)))
    }
}

// ---------------------------------------------------------------------------
// User 도메인
// ---------------------------------------------------------------------------

/// User 도메인 전용 트랜잭션 컨텍스트.
pub struct PgUserUnitOfWork {
    shared_tx: SharedTx,
    user_repo: PgUserRepo,
}

impl PgUserUnitOfWork {
    fn new(tx: Transaction<'static, Postgres>) -> Self {
        let shared_tx: SharedTx = Arc::new(Mutex::new(Some(tx)));
        Self {
            user_repo: PgUserRepo::new(shared_tx.clone()),
            shared_tx,
        }
    }
}

#[async_trait]
impl UserUnitOfWork for PgUserUnitOfWork {
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

/// User 도메인 UoW 팩토리.
pub struct PgUserUnitOfWorkFactory {
    pool: PgPool,
}

impl PgUserUnitOfWorkFactory {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserUnitOfWorkFactory for PgUserUnitOfWorkFactory {
    async fn begin(&self) -> anyhow::Result<Box<dyn UserUnitOfWork>> {
        let tx = self.pool.begin().await?;
        Ok(Box::new(PgUserUnitOfWork::new(tx)))
    }
}
