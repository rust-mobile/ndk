use crate::error::NdkError;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[repr(u8)]
pub enum Target {
    #[serde(rename = "armv7-linux-androideabi")]
    ArmV7a = 1,
    #[serde(rename = "aarch64-linux-android")]
    Arm64V8a = 2,
    #[serde(rename = "i686-linux-android")]
    X86 = 3,
    #[serde(rename = "x86_64-linux-android")]
    X86_64 = 4,
}

impl Target {
    /// Identifier used in the NDK to refer to the ABI
    pub fn android_abi(self) -> &'static str {
        match self {
            Self::Arm64V8a => "arm64-v8a",
            Self::ArmV7a => "armeabi-v7a",
            Self::X86 => "x86",
            Self::X86_64 => "x86_64",
        }
    }

    /// Returns `Target` for abi.
    pub fn from_android_abi(abi: &str) -> Result<Self, NdkError> {
        match abi {
            "arm64-v8a" => Ok(Self::Arm64V8a),
            "armeabi-v7a" => Ok(Self::ArmV7a),
            "x86" => Ok(Self::X86),
            "x86_64" => Ok(Self::X86_64),
            _ => Err(NdkError::UnsupportedTarget),
        }
    }

    /// Returns the triple used by the rust build tools
    pub fn rust_triple(self) -> &'static str {
        match self {
            Self::Arm64V8a => "aarch64-linux-android",
            Self::ArmV7a => "armv7-linux-androideabi",
            Self::X86 => "i686-linux-android",
            Self::X86_64 => "x86_64-linux-android",
        }
    }

    /// Returns `Target` for rust triple.
    pub fn from_rust_triple(triple: &str) -> Result<Self, NdkError> {
        match triple {
            "aarch64-linux-android" => Ok(Self::Arm64V8a),
            "armv7-linux-androideabi" => Ok(Self::ArmV7a),
            "i686-linux-android" => Ok(Self::X86),
            "x86_64-linux-android" => Ok(Self::X86_64),
            _ => Err(NdkError::UnsupportedTarget),
        }
    }

    // Returns the triple NDK provided LLVM
    pub fn ndk_llvm_triple(self) -> &'static str {
        match self {
            Self::Arm64V8a => "aarch64-linux-android",
            Self::ArmV7a => "armv7a-linux-androideabi",
            Self::X86 => "i686-linux-android",
            Self::X86_64 => "x86_64-linux-android",
        }
    }

    /// Returns the triple used by the non-LLVM parts of the NDK
    pub fn ndk_triple(self) -> &'static str {
        match self {
            Self::Arm64V8a => "aarch64-linux-android",
            Self::ArmV7a => "arm-linux-androideabi",
            Self::X86 => "i686-linux-android",
            Self::X86_64 => "x86_64-linux-android",
        }
    }
}
