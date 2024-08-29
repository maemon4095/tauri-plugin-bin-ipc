use std::marker::PhantomData;
use tauri::AppHandle;

/// AppHandleは外部クレートの型なので、tauri::command::CommandArgのようなtrait boundを使った処理の切り替えはできない。
///
/// HACK: methodルックアップの優先度を利用して疑似的にoverrideを実現している。
///       methodのルックアップはその型のメンバを探し、次にderefした型のメンバを探す。
///       そのため、deref先の型にデフォルト実装を用意し、元の型に特化実装を与えるとoverrideが再現できる。
///
/// SEE: https://doc.rust-lang.org/reference/expressions/method-call-expr.html
//

pub fn deserialize_proxy<R: tauri::Runtime, T>() -> DeserializeProxy<R, T> {
    DeserializeProxy(PhantomData)
}

pub struct DeserializeProxy<R: tauri::Runtime, T>(PhantomData<(R, T)>);
impl<R: tauri::Runtime, T: serde::de::DeserializeOwned> DeserializeProxy<R, T> {
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(&self, de: D) -> Result<T, D::Error> {
        T::deserialize(de)
    }
}

impl<R: tauri::Runtime, T> std::ops::Deref for DeserializeProxy<R, T> {
    type Target = DeserializeAppHandleProxy<R, T>;

    fn deref(&self) -> &Self::Target {
        &DeserializeAppHandleProxy(PhantomData)
    }
}

pub struct DeserializeAppHandleProxy<R: tauri::Runtime, T>(PhantomData<(R, T)>);

impl<R: tauri::Runtime> DeserializeAppHandleProxy<R, AppHandle<R>> {
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        &self,
        _de: D,
    ) -> Result<AppHandle<R>, D::Error> {
        Err(<D::Error as serde::de::Error>::custom(
            crate::error::ShouldNotBeDeserlialized::<AppHandle<R>>::new(),
        ))
    }
}
