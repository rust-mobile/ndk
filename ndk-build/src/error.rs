use std::io::Error as IoError;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NdkError {
    #[error(
        "Android SDK is not found. \
    Please set the path to the Android SDK with the $ANDROID_SDK_ROOT \
    environment variable."
    )]
    SdkNotFound,
    #[error(
        "Android NDK is not found. \
        Please set the path to the Android NDK with $ANDROID_NDK_ROOT \
        environment variable."
    )]
    NdkNotFound,
    #[error("GNU toolchain binary `{gnu_bin}` nor LLVM toolchain binary `{llvm_bin}` found in `{toolchain_path:?}`.")]
    ToolchainBinaryNotFound {
        toolchain_path: PathBuf,
        gnu_bin: String,
        llvm_bin: String,
    },
    #[error("Path `{0:?}` doesn't exist.")]
    PathNotFound(PathBuf),
    #[error("Command `{0}` not found.")]
    CmdNotFound(String),
    #[error("Android SDK has no build tools.")]
    BuildToolsNotFound,
    #[error("Android SDK has no platforms installed.")]
    NoPlatformFound,
    #[error("Platform `{0}` is not installed.")]
    PlatformNotFound(u32),
    #[error("Target is not supported.")]
    UnsupportedTarget,
    #[error("Host `{0}` is not supported.")]
    UnsupportedHost(String),
    #[error(transparent)]
    Io(#[from] IoError),
    #[error("Invalid semver")]
    InvalidSemver,
    #[error("Command `{}` had a non-zero exit code.", format!("{:?}", .0).replace('"', ""))]
    CmdFailed(Command),
    #[error(transparent)]
    Serialize(#[from] quick_xml::de::DeError),
}
