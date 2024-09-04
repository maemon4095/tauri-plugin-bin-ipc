use tauri::async_runtime::JoinHandle;

use crate::BinIpcError;

pub struct FlattenJoinHandle<T>(
    <JoinHandle<Result<T, BinIpcError>> as std::future::IntoFuture>::IntoFuture,
);

impl<T> std::future::Future for FlattenJoinHandle<T> {
    type Output = Result<T, BinIpcError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|f| &mut f.0) }
            .poll(cx)
            .map(|e| match e {
                Ok(Ok(v)) => Ok(v),
                Ok(Err(e)) => Err(e),
                Err(e) => Err(BinIpcError::from(e)),
            })
    }
}

impl<T> From<JoinHandle<Result<T, BinIpcError>>> for FlattenJoinHandle<T> {
    fn from(value: JoinHandle<Result<T, BinIpcError>>) -> Self {
        Self(value)
    }
}
