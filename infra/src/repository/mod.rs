pub mod health_check;
pub mod todo;
pub mod user;

pub struct UserRepositoryImpl {}

impl UserRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct TodoRepositoryImpl {}

impl TodoRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct TodoStatusRepositoryImpl {}

impl TodoStatusRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}
