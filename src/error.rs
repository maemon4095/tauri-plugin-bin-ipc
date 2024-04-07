#[derive(Debug, thiserror::Error)]
#[error("the request URI is invalid.")]
pub struct RequestPathError;

#[derive(Debug, thiserror::Error)]
#[error("bin-ipc request method must be POST")]
pub struct InvalidMethodError;

pub type BoxError = Box<dyn std::error::Error>;
