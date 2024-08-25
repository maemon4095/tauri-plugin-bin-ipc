mod builder_ext;
mod handler;
mod plugin_builder_ext;
mod protocol;
mod secure_arena;
mod util;

pub type BoxError = Box<dyn 'static + Send + std::error::Error>;
pub use builder_ext::BuilderBinIpcExtension;
pub use handler::BinIpcHandler;
pub use plugin_builder_ext::PluginBuilderBinIpcExtension;
