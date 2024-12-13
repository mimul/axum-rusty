use todo_domain::model::user::{NewUser, User};
use todo_domain::model::Id;

pub struct UserView {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

impl From<User> for UserView {
    fn from(u: User) -> Self {
        Self {
            id: u.id.value.to_string(),
            username: u.username,
            email: u.email,
            password: u.password,
        }
    }
}
pub struct CreateUser {
    pub username: String,
    pub password: String,
}

impl CreateUser {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}

impl TryFrom<CreateUser> for NewUser {
    type Error = anyhow::Error;

    fn try_from(cu: CreateUser) -> Result<Self, Self::Error> {
        Ok(NewUser::new(Id::gen(), cu.username, cu.password))
    }
}

pub struct SearchUserCondition {
    pub username: Option<String>,
}
