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
