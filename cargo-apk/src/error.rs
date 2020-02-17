use cargo_subcommand::Error as SubcommandError;
use ndk_build::error::NdkError;
use std::fmt::{Display, Formatter, Result};
use std::io::Error as IoError;
use toml::de::Error as TomlError;

#[derive(Debug)]
pub enum Error {
    Subcommand(SubcommandError),
    Config(TomlError),
    Ndk(NdkError),
    Io(IoError),
}

impl Error {
    pub fn invalid_args() -> Self {
        Self::Subcommand(SubcommandError::InvalidArgs)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::Subcommand(error) => error.fmt(f),
            Self::Config(error) => return write!(f, "Failed to parse config: {}.", error),
            Self::Ndk(error) => error.fmt(f),
            Self::Io(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<SubcommandError> for Error {
    fn from(error: SubcommandError) -> Self {
        Self::Subcommand(error)
    }
}

impl From<TomlError> for Error {
    fn from(error: TomlError) -> Self {
        Self::Config(error)
    }
}

impl From<NdkError> for Error {
    fn from(error: NdkError) -> Self {
        Self::Ndk(error)
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Self::Io(error)
    }
}
