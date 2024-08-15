mod handler;
mod plugin_build_ext;
mod secure_arena;
mod util;
type BoxError = Box<dyn 'static + Send + std::error::Error>;

pub use plugin_build_ext::PluginBuilderBinIpcExtension;
