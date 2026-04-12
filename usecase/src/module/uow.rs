use async_trait::async_trait;
use domain::repository::todo::status::TodoStatusRepository;
use domain::repository::todo::TodoRepository;
use domain::repository::user::UserRepository;

/// Todo 도메인 트랜잭션 컨텍스트.
///
/// Todo 관련 레포지토리만 노출한다.
/// `commit()` 없이 drop → sqlx Transaction drop → 자동 롤백.
#[async_trait]
pub trait TodoUnitOfWork: Send {
    fn todo_repo(&self) -> &dyn TodoRepository;
    fn todo_status_repo(&self) -> &dyn TodoStatusRepository;
    async fn commit(&mut self) -> anyhow::Result<()>;
    async fn rollback(&mut self) -> anyhow::Result<()>;
}

/// User 도메인 트랜잭션 컨텍스트.
///
/// User 관련 레포지토리만 노출한다.
/// `commit()` 없이 drop → sqlx Transaction drop → 자동 롤백.
#[async_trait]
pub trait UserUnitOfWork: Send {
    fn user_repo(&self) -> &dyn UserRepository;
    async fn commit(&mut self) -> anyhow::Result<()>;
    async fn rollback(&mut self) -> anyhow::Result<()>;
}

/// Todo 도메인 UoW 팩토리 포트.
///
/// 구현체는 `infra` 크레이트의 `PgTodoUnitOfWorkFactory`.
#[async_trait]
pub trait TodoUnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> anyhow::Result<Box<dyn TodoUnitOfWork>>;
}

/// User 도메인 UoW 팩토리 포트.
///
/// 구현체는 `infra` 크레이트의 `PgUserUnitOfWorkFactory`.
#[async_trait]
pub trait UserUnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> anyhow::Result<Box<dyn UserUnitOfWork>>;
}
