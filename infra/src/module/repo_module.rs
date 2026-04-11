use crate::repository::todo::status::TodoStatusRepositoryImpl;
use crate::repository::todo::TodoRepositoryImpl;
use crate::repository::user::UserRepositoryImpl;
use usecase::module::repos::RepositoriesModuleExt;

pub struct RepositoriesModule {
    user_repository: UserRepositoryImpl,
    todo_repository: TodoRepositoryImpl,
    todo_status_repository: TodoStatusRepositoryImpl,
}

impl RepositoriesModuleExt for RepositoriesModule {
    type UserRepo = UserRepositoryImpl;
    type TodoRepo = TodoRepositoryImpl;
    type TodoStatusRepo = TodoStatusRepositoryImpl;

    fn user_repository(&self) -> &Self::UserRepo {
        &self.user_repository
    }
    fn todo_repository(&self) -> &Self::TodoRepo {
        &self.todo_repository
    }
    fn todo_status_repository(&self) -> &Self::TodoStatusRepo {
        &self.todo_status_repository
    }
}

impl Default for RepositoriesModule {
    fn default() -> Self {
        Self::new()
    }
}

impl RepositoriesModule {
    pub fn new() -> Self {
        let user_repository = UserRepositoryImpl::new();
        let todo_repository = TodoRepositoryImpl::new();
        let todo_status_repository = TodoStatusRepositoryImpl::new();
        Self {
            user_repository,
            todo_repository,
            todo_status_repository,
        }
    }
}
