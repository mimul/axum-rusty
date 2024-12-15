use crate::model::Id;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Id<User>,
    pub username: String,
    pub email: String,
    pub password: String,
    pub fullname: String,
}

impl User {
    pub fn new(id: Id<User>, username: String, email: String, password: String, fullname: String) -> Self {
        Self {
            id,
            username,
            email,
            password,
            fullname,
        }
    }
}

pub struct NewUser {
    pub id: Id<User>,
    pub username: String,
    pub password: String,
    pub fullname: String,
}

impl NewUser {
    pub fn new(id: Id<User>, username: String, password: String, fullname: String) -> Self {
        Self {
            id,
            username,
            password,
            fullname,
        }
    }
}
