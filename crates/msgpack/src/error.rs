#[derive(Debug)]
pub struct MissingArgumentError {
    pub command_name: &'static str,
    pub arg_name: &'static str,
}

impl std::fmt::Display for MissingArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "missing argument `{}` of command `{}`",
            self.arg_name, self.command_name
        )
    }
}

impl std::error::Error for MissingArgumentError {}

// pub type BoxError = Box<dyn std::error::Error + Send>;

#[derive(Debug)]
pub struct BoxError(Box<dyn std::error::Error + Send>);

impl std::fmt::Display for BoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: 'static + std::error::Error + Send> From<T> for BoxError {
    fn from(value: T) -> Self {
        Self(Box::new(value))
    }
}

impl BoxError {
    pub fn new<T: 'static + std::error::Error + Send>(e: T) -> Self {
        Self(Box::new(e))
    }
}
