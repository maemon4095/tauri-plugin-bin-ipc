use crate::{handler::BinIpcHandler, protocol};
use serde::de::DeserializeOwned;
use tauri::plugin;

pub trait PluginBuilderBinIpcExtension<R: tauri::Runtime> {
    fn bin_ipc_handler<H: BinIpcHandler<R>>(self, scheme: impl Into<String>, handler: H) -> Self;
}

impl<R, C> PluginBuilderBinIpcExtension<R> for plugin::Builder<R, C>
where
    R: tauri::Runtime,
    C: DeserializeOwned,
{
    fn bin_ipc_handler<H: BinIpcHandler<R>>(self, scheme: impl Into<String>, handler: H) -> Self {
        let scheme = scheme.into();
        self.register_uri_scheme_protocol(scheme.clone(), protocol::create(&scheme, handler))
    }
}
