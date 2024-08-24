pub use core::{BinIpcHandler, BoxError, PluginBuilderBinIpcExtension};
#[cfg(feature = "msgpack")]
pub use msgpack;
#[cfg(feature = "default-msgpack")]
pub use msgpack::{bin_command, generate_bin_handler};
