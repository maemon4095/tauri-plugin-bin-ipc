pub use crate::{
    de::deserialize_proxy,
    error::{MissingArgumentError, NoSuchCommandError},
    flatten_join_handle::FlattenJoinHandle,
    or_future::OrFuture,
    wrap_result::wrap_result,
    HandleResult, TauriPluginBinIpcMessagePackCommand,
};

pub use bin_ipc_core::{BinIpcHandler, BoxError};
pub use bin_ipc_util::{from_app_handle_proxy, FromAppHandle};
pub use rmp_serde::{
    decode::Error as MsgpackDecodeError, to_vec as encode_to_vec,
    Deserializer as MsgpackDeserializer,
};
pub use serde::{
    de::{DeserializeSeed, Error as SerdeError, MapAccess, Visitor},
    Deserializer,
};
pub use std::{
    self,
    error::Error as StdError,
    fmt::{
        Debug as StdDebug, Display as StdDisplay, Formatter as StdFormatter, Result as StdFmtResult,
    },
    future::{ready as ready_future, Future},
    marker::PhantomData,
    prelude::rust_2021::*,
    primitive::*,
};
pub use tauri::{
    async_runtime::{spawn, spawn_blocking, JoinHandle as TauriJoinHandle},
    AppHandle, Runtime,
};
