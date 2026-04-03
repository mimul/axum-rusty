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
    fn from(user: User) -> Self {
        Self {
            id: user.id.value.to_string(),
            username: user.username,
            email: user.email,
            password: user.password,
            fullname: user.fullname,
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

impl LoginUser {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password, }
    }
}

pub struct SearchUserCondition {
    pub username: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::model::Id;

    fn make_user() -> (domain::model::user::User, String) {
        let id: Id<domain::model::user::User> = Id::gen();
        let ulid_str = id.value.to_string();
        let user = domain::model::user::User::new(
            id,
            "alice".to_string(),
            "alice@example.com".to_string(),
            "hashed".to_string(),
            "Alice".to_string(),
        );
        (user, ulid_str)
    }

    #[test]
    fn user_view_from_user_maps_all_fields() {
        let (user, ulid_str) = make_user();
        let view = UserView::from(user);
        assert_eq!(view.id, ulid_str);
        assert_eq!(view.username, "alice");
        assert_eq!(view.email, "alice@example.com");
        assert_eq!(view.password, "hashed");
        assert_eq!(view.fullname, "Alice");
    }

    #[test]
    fn create_user_try_into_new_user_generates_id() {
        let cu = CreateUser::new("bob".to_string(), "pw".to_string(), "Bob".to_string());
        let nu: NewUser = cu.try_into().unwrap();
        assert_eq!(nu.username, "bob");
        assert_eq!(nu.password, "pw");
        assert_eq!(nu.fullname, "Bob");
    }
}
