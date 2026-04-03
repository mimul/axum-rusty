use anyhow::anyhow;
use std::marker::PhantomData;
use ulid::Ulid;

pub mod todo;
pub mod user;

#[derive(Debug, Clone, Copy)]
pub struct Id<T> {
    pub value: Ulid,
    _marker: PhantomData<T>,
}

impl<T> Id<T> {
    pub fn new(value: Ulid) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    pub fn gen() -> Id<T> {
        Id::new(Ulid::new())
    }
}

impl<T> TryFrom<String> for Id<T> {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ulid::from_string(&value)
            .map(|id| Self::new(id))
            .map_err(|err| anyhow!("{:?}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy;

    #[test]
    fn id_gen_creates_unique_ids() {
        let id1 = Id::<Dummy>::gen();
        let id2 = Id::<Dummy>::gen();
        assert_ne!(id1.value, id2.value);
    }

    #[test]
    fn id_try_from_valid_ulid_string_succeeds() {
        let ulid = Ulid::new();
        let result = Id::<Dummy>::try_from(ulid.to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, ulid);
    }

    #[test]
    fn id_try_from_invalid_string_returns_error() {
        let result = Id::<Dummy>::try_from("not-a-valid-ulid".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn id_try_from_empty_string_returns_error() {
        let result = Id::<Dummy>::try_from("".to_string());
        assert!(result.is_err());
    }
}
