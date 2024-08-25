use std::marker::PhantomData;

pub struct WrapResult<T>(pub PhantomData<T>);

impl<T, E> WrapResult<Result<T, E>> {
    pub fn wrap(&self, value: Result<T, E>) -> Result<T, E> {
        value
    }
}

impl<T> std::ops::Deref for WrapResult<T> {
    type Target = WrapResultNeverFail<T>;

    fn deref(&self) -> &Self::Target {
        &WrapResultNeverFail(PhantomData)
    }
}

pub struct WrapResultNeverFail<T>(PhantomData<T>);

#[derive(Debug)]
pub enum NeverFail {}

impl std::fmt::Display for NeverFail {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}

impl std::error::Error for NeverFail {}

impl<T> WrapResultNeverFail<T> {
    pub fn wrap(&self, value: T) -> Result<T, NeverFail> {
        Ok(value)
    }
}
