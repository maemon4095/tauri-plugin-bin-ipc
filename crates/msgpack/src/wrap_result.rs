use std::marker::PhantomData;

pub struct WrapResult<T>(pub PhantomData<T>);

mod private {
    pub trait IsResult {
        type Ok;
        type Err;
        fn into_result(self) -> Result<Self::Ok, Self::Err>;
    }
}

impl<T, E> private::IsResult for Result<T, E> {
    type Ok = T;
    type Err = E;

    fn into_result(self) -> Result<T, E> {
        self
    }
}

impl<T: private::IsResult> WrapResult<T> {
    pub fn wrap(&self, value: T) -> Result<T::Ok, T::Err> {
        value.into_result()
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
