use domain::model::user::{NewUser, User};
use sqlx::FromRow;

#[derive(FromRow, Debug)]
pub struct StoredUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub fullname: String,
}

impl TryFrom<StoredUser> for User {
    type Error = anyhow::Error;

    fn try_from(u: StoredUser) -> Result<Self, Self::Error> {
        Ok(User {
            id: u.id.try_into()?,
            username: u.username,
            email: u.email,
            password: u.password,
            fullname: u.fullname,
        })
    }
}

#[derive(FromRow, Debug)]
pub struct InsertUser {
    pub id: String,
    pub username: String,
    pub password: String,
    pub fullname: String,
}

impl From<NewUser> for InsertUser {
    fn from(nu: NewUser) -> Self {
        InsertUser {
            id: nu.id.value.to_string(),
            username: nu.username,
            password: nu.password,
            fullname: nu.fullname,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::model::Id;

    #[test]
    fn insert_user_from_new_user_maps_all_fields() {
        let id: Id<domain::model::user::User> = Id::gen();
        let ulid_str = id.value.to_string();
        let nu = NewUser::new(
            id,
            "charlie".to_string(),
            "pw".to_string(),
            "Charlie".to_string(),
        );
        let insert: InsertUser = nu.into();
        assert_eq!(insert.id, ulid_str);
        assert_eq!(insert.username, "charlie");
        assert_eq!(insert.password, "pw");
        assert_eq!(insert.fullname, "Charlie");
    }

    #[test]
    fn stored_user_try_into_user_succeeds_with_valid_ulid() {
        let id: Id<domain::model::user::User> = Id::gen();
        let ulid_str = id.value.to_string();
        let ulid = id.value;
        let stored = StoredUser {
            id: ulid_str,
            username: "dave".to_string(),
            email: "dave@example.com".to_string(),
            password: "hashed".to_string(),
            fullname: "Dave".to_string(),
        };
        let user: User = stored.try_into().unwrap();
        assert_eq!(user.id.value, ulid);
        assert_eq!(user.username, "dave");
        assert_eq!(user.email, "dave@example.com");
    }

    #[test]
    fn stored_user_try_into_user_fails_with_invalid_id() {
        let stored = StoredUser {
            id: "not-a-ulid".to_string(),
            username: "eve".to_string(),
            email: "eve@example.com".to_string(),
            password: "pw".to_string(),
            fullname: "Eve".to_string(),
        };
        let result: Result<User, _> = stored.try_into();
        assert!(result.is_err());
    }
}
