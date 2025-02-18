use domain::repository::todo::status::TodoStatusRepository;
use domain::repository::todo::TodoRepository;
use domain::repository::user::UserRepository;
use crate::repository::todo::status::TodoStatusRepositoryImpl;
use crate::repository::todo::TodoRepositoryImpl;
use crate::repository::user::UserRepositoryImpl;

pub struct RepositoriesModule {
    user_repository: UserRepositoryImpl,
    todo_repository: TodoRepositoryImpl,
    todo_status_repository: TodoStatusRepositoryImpl,
}

pub trait RepositoriesModuleExt {
    type UserRepo: UserRepository;
    type TodoRepo: TodoRepository;
    type TodoStatusRepo: TodoStatusRepository;

    fn user_repository(&self) -> &Self::UserRepo;
    fn todo_repository(&self) -> &Self::TodoRepo;
    fn todo_status_repository(&self) -> &Self::TodoStatusRepo;
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

