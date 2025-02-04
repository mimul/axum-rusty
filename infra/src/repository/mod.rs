use std::marker::PhantomData;

pub mod health_check;
pub mod todo;
pub mod user;

pub struct DatabaseRepositoryImpl<T> {
    _marker: PhantomData<T>,
}

impl<T> DatabaseRepositoryImpl<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
