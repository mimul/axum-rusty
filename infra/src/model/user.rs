use sqlx::FromRow;
use domain::model::user::{NewUser, User};

#[derive(FromRow, Debug)]
pub struct StoredUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

impl TryFrom<StoredUser> for User {
    type Error = anyhow::Error;

    fn try_from(u: StoredUser) -> Result<Self, Self::Error> {
        Ok(User {
            id: u.id.try_into()?,
            username: u.username,
            email: u.email,
            password: u.password,
        })
    }
}

#[derive(FromRow, Debug)]
pub struct InsertUser {
    pub id: String,
    pub username: String,
    pub password: String,
}

impl From<NewUser> for InsertUser {
    fn from(nu: NewUser) -> Self {
        InsertUser {
            id: nu.id.value.to_string(),
            username: nu.username,
            password: nu.password,
        }
    }
}
