mod channel;
mod channel_credentials;
mod connection;
mod error;
mod event_emitter;
mod listener;
mod plugin_builder_bin_ipc_extension;
mod request_path;
mod state;

use listener::Listener;
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Runtime,
};

pub use channel::{Receiver, Sender};
pub use plugin_builder_bin_ipc_extension::PluginBuilderBinIpcExtension;

type Body = Vec<u8>;

// TODO: TcpListenerと同じInterfaceを加える。
// ```rust
// let listener = bind(80).await?;
// let conn = listener.accept().await?;
// ```
//
// ```ts
// const conn = await connect({ port: 80 });
// ```

pub struct Builder<R: Runtime> {
    inits: Vec<Box<dyn FnOnce(PluginBuilder<R>) -> PluginBuilder<R>>>,
}

impl<R: Runtime> Default for Builder<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: Runtime> Builder<R> {
    pub fn new() -> Self {
        Self { inits: Vec::new() }
    }

    pub fn register_bin_ipc_protocol<L: Listener<R>>(
        mut self,
        scheme: impl Into<String>,
        listener: L,
    ) -> Self {
        let scheme = scheme.into();
        self.inits.push(Box::new(move |builder| {
            builder.register_bin_ipc_protocol(scheme, listener)
        }));

        self
    }

    pub fn build(self) -> TauriPlugin<R> {
        let mut builder = PluginBuilder::new("bin-ipc");
        for init in self.inits {
            builder = init(builder);
        }
        builder.build()
    }
}
