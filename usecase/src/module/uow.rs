use async_trait::async_trait;
use domain::repository::todo::status::TodoStatusRepository;
use domain::repository::todo::TodoRepository;
use domain::repository::user::UserRepository;

/// 단일 트랜잭션 컨텍스트.
///
/// `commit()` 호출 전까지 모든 작업은 자동 롤백된다.
/// 에러 전파(`?`) 시 UoW가 drop → sqlx Transaction이 drop → 자동 롤백.
#[async_trait]
pub trait UnitOfWork: Send {
    fn todo_repo(&self) -> &dyn TodoRepository;
    fn todo_status_repo(&self) -> &dyn TodoStatusRepository;
    fn user_repo(&self) -> &dyn UserRepository;
    async fn commit(&mut self) -> anyhow::Result<()>;
    async fn rollback(&mut self) -> anyhow::Result<()>;
}

/// `UnitOfWork` 인스턴스를 생성하는 팩토리 포트.
///
/// 구현체는 `infra` 크레이트의 `PgUnitOfWorkFactory`가 담당한다.
/// 의존성 방향: infra → usecase (인터페이스 소유권이 사용처에 있음)
#[async_trait]
pub trait UnitOfWorkFactory: Send + Sync {
    async fn begin(&self) -> anyhow::Result<Box<dyn UnitOfWork>>;
}
