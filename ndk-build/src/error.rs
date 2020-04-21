use std::error::Error;
use std::fmt::{Display, Formatter, Result};
use std::io::Error as IoError;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub enum NdkError {
    SdkNotFound,
    NdkNotFound,
    PathNotFound(PathBuf),
    CmdNotFound(String),
    CmdFailed(Command),
    BuildToolsNotFound,
    NoPlatformFound,
    PlatformNotFound(u32),
    UnsupportedTarget,
    UnsupportedHost(String),
    Io(IoError),
    InvalidSemver,
}

impl Display for NdkError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let msg = match self {
            Self::SdkNotFound => {
                "Please set the path to the Android SDK with either the $ANDROID_SDK_HOME or \
                 the $ANDROID_HOME environment variable."
            }
            Self::NdkNotFound => {
                "Please set the path to the Android NDK with either the $ANDROID_NDK_HOME or \
                 the $NDK_HOME environment variable."
            }
            Self::PathNotFound(path) => return write!(f, "Path {:?} doesn't exist.", path),
            Self::CmdNotFound(cmd) => return write!(f, "Command {} not found.", cmd),
            Self::CmdFailed(cmd) => {
                let cmd = format!("{:?}", cmd).replace('"', "");
                return write!(f, "Command '{}' had a non-zero exit code.", cmd);
            }
            Self::BuildToolsNotFound => "Android SDK has no build tools.",
            Self::NoPlatformFound => "Android SDK has no platforms installed.",
            Self::PlatformNotFound(level) => {
                return write!(f, "Platform {} is not installed.", level)
            }
            Self::UnsupportedTarget => "Target is not supported.",
            Self::UnsupportedHost(host) => return write!(f, "Host {} is not supported.", host),
            Self::Io(error) => return error.fmt(f),
            Self::InvalidSemver => return write!(f, "Invalid semver"),
        };
        msg.fmt(f)
    }
}

impl Error for NdkError {}

impl From<IoError> for NdkError {
    fn from(error: IoError) -> Self {
        Self::Io(error)
    }
}
