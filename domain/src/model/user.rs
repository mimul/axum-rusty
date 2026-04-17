use crate::model::Id;

#[derive(Clone)]
pub struct User {
    pub id: Id<User>,
    pub username: String,
    pub email: String,
    pub password: String,
    pub fullname: String,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("email", &self.email)
            .field("password", &"****")
            .field("fullname", &self.fullname)
            .finish()
    }
}

impl User {
    pub fn new(
        id: Id<User>,
        username: String,
        email: String,
        password: String,
        fullname: String,
    ) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Id;
    use ulid::Ulid;

    #[test]
    fn user_new_stores_all_fields() {
        let ulid = Ulid::new();
        let user = User::new(
            Id::new(ulid),
            "alice".to_string(),
            "alice@example.com".to_string(),
            "hashed_pw".to_string(),
            "Alice Smith".to_string(),
        );
        assert_eq!(user.id.value, ulid);
        assert_eq!(user.username, "alice");
        assert_eq!(user.email, "alice@example.com");
        assert_eq!(user.password, "hashed_pw");
        assert_eq!(user.fullname, "Alice Smith");
    }

    #[test]
    fn new_user_new_stores_all_fields() {
        let ulid = Ulid::new();
        let nu = NewUser::new(
            Id::new(ulid),
            "bob".to_string(),
            "secret".to_string(),
            "Bob Jones".to_string(),
        );
        assert_eq!(nu.id.value, ulid);
        assert_eq!(nu.username, "bob");
        assert_eq!(nu.password, "secret");
        assert_eq!(nu.fullname, "Bob Jones");
    }

    #[test]
    fn user_debug_masks_password() {
        let ulid = Ulid::new();
        let user = User::new(
            Id::new(ulid),
            "alice".to_string(),
            "alice@example.com".to_string(),
            "secret_hash".to_string(),
            "Alice Smith".to_string(),
        );
        let debug_str = format!("{:?}", user);
        assert!(!debug_str.contains("secret_hash"));
        assert!(debug_str.contains("****"));
    }
}
