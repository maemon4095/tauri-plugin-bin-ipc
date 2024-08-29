use super::FromAppHandle;
use std::marker::PhantomData;
use tauri::AppHandle;

pub struct FromAppHandleProxy<R, T>(PhantomData<(R, T)>);

pub fn from_app_handle_proxy<R, T>() -> FromAppHandleProxy<R, T> {
    FromAppHandleProxy(PhantomData)
}

impl<R: tauri::Runtime, T: FromAppHandle<R>> FromAppHandleProxy<R, T> {
    pub fn from_app_handle(&self, app: &AppHandle<R>) -> Option<T> {
        Some(T::from_app_handle(app))
    }
}

impl<R: tauri::Runtime, T> std::ops::Deref for FromAppHandleProxy<R, T> {
    type Target = FromAppHandleProxyFallback<R, T>;

    fn deref(&self) -> &Self::Target {
        &FromAppHandleProxyFallback(PhantomData)
    }
}

pub struct FromAppHandleProxyFallback<R, T>(PhantomData<(R, T)>);

impl<R: tauri::Runtime, T> FromAppHandleProxyFallback<R, T> {
    pub fn from_app_handle(&self, _: &AppHandle<R>) -> Option<T> {
        None
    }
}
