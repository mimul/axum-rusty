use crate::repository::DatabaseRepositoryImpl;
use domain::model::todo::status::TodoStatus;
use domain::model::todo::Todo;
use domain::model::user::User;
use domain::repository::todo::status::TodoStatusRepository;
use domain::repository::todo::TodoRepository;
use domain::repository::user::UserRepository;

pub struct RepositoriesModule {
    user_repository: DatabaseRepositoryImpl<User>,
    todo_repository: DatabaseRepositoryImpl<Todo>,
    todo_status_repository: DatabaseRepositoryImpl<TodoStatus>,
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
    type UserRepo = DatabaseRepositoryImpl<User>;
    type TodoRepo = DatabaseRepositoryImpl<Todo>;
    type TodoStatusRepo = DatabaseRepositoryImpl<TodoStatus>;

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
        let user_repository = DatabaseRepositoryImpl::new();
        let todo_repository = DatabaseRepositoryImpl::new();
        let todo_status_repository = DatabaseRepositoryImpl::new();
        Self {
            user_repository,
            todo_repository,
            todo_status_repository,
        }
    }
}
