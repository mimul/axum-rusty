use domain::model::user::{NewUser, User};
use domain::model::Id;

#[derive(Debug, Clone)]
pub struct UserView {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub fullname: String,
}

impl From<User> for UserView {
    fn from(u: User) -> Self {
        Self {
            id: u.id.value.to_string(),
            username: u.username,
            email: u.email,
            password: u.password,
            fullname: u.fullname,
        }
    }
}
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub fullname: String,
}

impl CreateUser {
    pub fn new(username: String, password: String, fullname: String) -> Self {
        Self { username, password, fullname }
    }
}

impl TryFrom<CreateUser> for NewUser {
    type Error = anyhow::Error;

    fn try_from(cu: CreateUser) -> Result<Self, Self::Error> {
        Ok(NewUser::new(Id::gen(), cu.username, cu.password, cu.fullname))
    }
}

pub struct LoginUser {
    pub username: String,
    pub password: String,
}

impl crate::model::user::LoginUser {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password, }
    }
}

pub struct SearchUserCondition {
    pub username: Option<String>,
}
