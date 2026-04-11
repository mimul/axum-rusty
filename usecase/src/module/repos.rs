use domain::repository::todo::status::TodoStatusRepository;
use domain::repository::todo::TodoRepository;
use domain::repository::user::UserRepository;

/// 유스케이스 레이어가 필요로 하는 리포지토리 컬렉션 인터페이스.
///
/// 구현체는 `infra` 크레이트의 `RepositoriesModule`이 담당한다.
/// 의존성 방향: infra → usecase (인터페이스 소유권이 사용처에 있음)
pub trait RepositoriesModuleExt {
    type UserRepo: UserRepository;
    type TodoRepo: TodoRepository;
    type TodoStatusRepo: TodoStatusRepository;

    fn user_repository(&self) -> &Self::UserRepo;
    fn todo_repository(&self) -> &Self::TodoRepo;
    fn todo_status_repository(&self) -> &Self::TodoStatusRepo;
}
