use std::marker::PhantomData;

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

#[derive(Debug)]
pub struct NoSuchCommandError(pub String);

impl std::fmt::Display for NoSuchCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Command `")?;
        f.write_str(&self.0)?;
        f.write_str("` does not exists.")
    }
}

impl std::error::Error for NoSuchCommandError {}

pub struct ShouldNotBeDeserlialized<T>(PhantomData<T>);

impl<T> ShouldNotBeDeserlialized<T> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> std::fmt::Debug for ShouldNotBeDeserlialized<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ShouldNotBeDeserlialized")
            .field(&std::any::type_name::<T>())
            .finish()
    }
}

impl<T> std::fmt::Display for ShouldNotBeDeserlialized<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The type `{}` should not be deserlialized.",
            std::any::type_name::<T>()
        )
    }
}

impl<T> std::error::Error for ShouldNotBeDeserlialized<T> {}
