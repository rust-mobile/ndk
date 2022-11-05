use cargo_subcommand::Error as SubcommandError;
use ndk_build::error::NdkError;
use std::io::Error as IoError;
use thiserror::Error;
use toml::de::Error as TomlError;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Subcommand(#[from] SubcommandError),
    #[error("Failed to parse config.")]
    Config(#[from] TomlError),
    #[error(transparent)]
    Ndk(#[from] NdkError),
    #[error(transparent)]
    Io(#[from] IoError),
    #[error("Configure a release keystore via `[package.metadata.android.signing.{0}]`")]
    MissingReleaseKey(String),
    #[error("`workspace=false` is unsupported")]
    InheritedFalse,
    #[error("`workspace=true` requires a workspace")]
    InheritanceMissingWorkspace,
    #[error("Failed to inherit field: `workspace.{0}` was not defined in workspace root manifest")]
    WorkspaceMissingInheritedField(&'static str),
}

impl Error {
    pub fn invalid_args() -> Self {
        Self::Subcommand(SubcommandError::InvalidArgs)
    }
}
