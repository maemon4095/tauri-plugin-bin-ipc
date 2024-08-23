use tauri::async_runtime::JoinHandle;

use crate::error::BoxError;

pub struct FlattenJoinHandle<T>(
    <JoinHandle<Result<T, BoxError>> as std::future::IntoFuture>::IntoFuture,
);

impl<T> std::future::Future for FlattenJoinHandle<T> {
    type Output = Result<T, BoxError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|f| &mut f.0) }
            .poll(cx)
            .map(|e| match e {
                Ok(Ok(v)) => Ok(v),
                Ok(Err(e)) => Err(e),
                Err(e) => Err(BoxError::new(e)),
            })
    }
}

impl<T> From<JoinHandle<Result<T, BoxError>>> for FlattenJoinHandle<T> {
    fn from(value: JoinHandle<Result<T, BoxError>>) -> Self {
        Self(value)
    }
}
