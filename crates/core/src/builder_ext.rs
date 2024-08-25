use crate::{handler::BinIpcHandler, protocol};

pub trait BuilderBinIpcExtension<R: tauri::Runtime> {
    fn bin_ipc_handler<H: BinIpcHandler<R>>(self, scheme: impl Into<String>, handler: H) -> Self;
}

impl<R> BuilderBinIpcExtension<R> for tauri::Builder<R>
where
    R: tauri::Runtime,
{
    fn bin_ipc_handler<H: BinIpcHandler<R>>(self, scheme: impl Into<String>, handler: H) -> Self {
        let scheme = scheme.into();
        self.register_uri_scheme_protocol(scheme.clone(), protocol::create(&scheme, handler))
    }
}
