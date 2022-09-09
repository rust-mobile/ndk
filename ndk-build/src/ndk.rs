use crate::error::NdkError;
use crate::target::Target;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ndk {
    sdk_path: PathBuf,
    ndk_path: PathBuf,
    build_tools_version: String,
    build_tag: u32,
    platforms: Vec<u32>,
}

impl Ndk {
    pub fn from_env() -> Result<Self, NdkError> {
        let sdk_path = {
            let mut sdk_path = std::env::var("ANDROID_HOME").ok();
            if sdk_path.is_some() {
                println!(
                    "Warning: You use environment variable ANDROID_HOME that is deprecated. \
                 Please, remove it and use ANDROID_SDK_ROOT instead. Now ANDROID_HOME is used"
                );
            }
            if sdk_path.is_none() {
                sdk_path = std::env::var("ANDROID_SDK_ROOT").ok();
            }

            PathBuf::from(sdk_path.ok_or(NdkError::SdkNotFound)?)
        };

        let ndk_path = {
            let ndk_path = std::env::var("ANDROID_NDK_ROOT")
                .ok()
                .or_else(|| std::env::var("ANDROID_NDK_PATH").ok())
                .or_else(|| std::env::var("ANDROID_NDK_HOME").ok())
                .or_else(|| std::env::var("NDK_HOME").ok());

            // default ndk installation path
            if ndk_path.is_none() && sdk_path.join("ndk-bundle").exists() {
                sdk_path.join("ndk-bundle")
            } else {
                PathBuf::from(ndk_path.ok_or(NdkError::NdkNotFound)?)
            }
        };

        let build_tools_dir = sdk_path.join("build-tools");
        let build_tools_version = std::fs::read_dir(&build_tools_dir)
            .or(Err(NdkError::PathNotFound(build_tools_dir)))?
            .filter_map(|path| path.ok())
            .filter(|path| path.path().is_dir())
            .filter_map(|path| path.file_name().into_string().ok())
            .filter(|name| name.chars().next().unwrap().is_ascii_digit())
            .max()
            .ok_or(NdkError::BuildToolsNotFound)?;

        let build_tag = std::fs::read_to_string(ndk_path.join("source.properties"))
            .expect("Failed to read source.properties");

        let build_tag = build_tag
            .split('\n')
            .find_map(|line| {
                let (key, value) = line
                    .split_once('=')
                    .expect("Failed to parse `key = value` from source.properties");
                if key.trim() == "Pkg.Revision" {
                    // AOSP writes a constantly-incrementing build version to the patch field.
                    // This number is incrementing across NDK releases.
                    let mut parts = value.trim().split('.');
                    let _major = parts.next().unwrap();
                    let _minor = parts.next().unwrap();
                    let patch = parts.next().unwrap();
                    // Can have an optional `XXX-beta1`
                    let patch = patch.split_once('-').map_or(patch, |(patch, _beta)| patch);
                    Some(patch.parse().expect("Failed to parse patch field"))
                } else {
                    None
                }
            })
            .expect("No `Pkg.Revision` in source.properties");

        let ndk_platforms = std::fs::read_to_string(ndk_path.join("build/core/platforms.mk"))?;
        let ndk_platforms = ndk_platforms
            .split('\n')
            .map(|s| s.split_once(" := ").unwrap())
            .collect::<HashMap<_, _>>();

        let min_platform_level = ndk_platforms["NDK_MIN_PLATFORM_LEVEL"]
            .parse::<u32>()
            .unwrap();
        let max_platform_level = ndk_platforms["NDK_MAX_PLATFORM_LEVEL"]
            .parse::<u32>()
            .unwrap();

        let platforms_dir = sdk_path.join("platforms");
        let platforms: Vec<u32> = std::fs::read_dir(&platforms_dir)
            .or(Err(NdkError::PathNotFound(platforms_dir)))?
            .filter_map(|path| path.ok())
            .filter(|path| path.path().is_dir())
            .filter_map(|path| path.file_name().into_string().ok())
            .filter_map(|name| {
                name.strip_prefix("android-")
                    .and_then(|api| api.parse::<u32>().ok())
            })
            .filter(|level| (min_platform_level..=max_platform_level).contains(level))
            .collect();

        if platforms.is_empty() {
            return Err(NdkError::NoPlatformFound);
        }

        Ok(Self {
            sdk_path,
            ndk_path,
            build_tools_version,
            build_tag,
            platforms,
        })
    }

    pub fn sdk(&self) -> &Path {
        &self.sdk_path
    }

    pub fn ndk(&self) -> &Path {
        &self.ndk_path
    }

    pub fn build_tools_version(&self) -> &str {
        &self.build_tools_version
    }

    pub fn build_tag(&self) -> u32 {
        self.build_tag
    }

    pub fn platforms(&self) -> &[u32] {
        &self.platforms
    }

    pub fn build_tool(&self, tool: &str) -> Result<Command, NdkError> {
        let path = self
            .sdk_path
            .join("build-tools")
            .join(&self.build_tools_version)
            .join(tool);
        if !path.exists() {
            return Err(NdkError::CmdNotFound(tool.to_string()));
        }
        Ok(Command::new(dunce::canonicalize(path)?))
    }

    pub fn platform_tool_path(&self, tool: &str) -> Result<PathBuf, NdkError> {
        let path = self.sdk_path.join("platform-tools").join(tool);
        if !path.exists() {
            return Err(NdkError::CmdNotFound(tool.to_string()));
        }
        Ok(dunce::canonicalize(path)?)
    }

    pub fn adb_path(&self) -> Result<PathBuf, NdkError> {
        self.platform_tool_path(bin!("adb"))
    }

    pub fn platform_tool(&self, tool: &str) -> Result<Command, NdkError> {
        Ok(Command::new(self.platform_tool_path(tool)?))
    }

    pub fn highest_supported_platform(&self) -> u32 {
        self.platforms().iter().max().cloned().unwrap()
    }

    /// Returns platform `30` as currently [required by Google Play], or lower
    /// when the detected SDK does not support it yet.
    ///
    /// [required by Google Play]: https://developer.android.com/distribute/best-practices/develop/target-sdk
    pub fn default_target_platform(&self) -> u32 {
        self.highest_supported_platform().min(30)
    }

    pub fn platform_dir(&self, platform: u32) -> Result<PathBuf, NdkError> {
        let dir = self
            .sdk_path
            .join("platforms")
            .join(format!("android-{}", platform));
        if !dir.exists() {
            return Err(NdkError::PlatformNotFound(platform));
        }
        Ok(dir)
    }

    pub fn android_jar(&self, platform: u32) -> Result<PathBuf, NdkError> {
        let android_jar = self.platform_dir(platform)?.join("android.jar");
        if !android_jar.exists() {
            return Err(NdkError::PathNotFound(android_jar));
        }
        Ok(android_jar)
    }

    fn host_arch() -> Result<&'static str, NdkError> {
        let host_os = std::env::var("HOST").ok();
        let host_contains = |s| host_os.as_ref().map(|h| h.contains(s)).unwrap_or(false);

        Ok(if host_contains("linux") {
            "linux"
        } else if host_contains("macos") {
            "darwin"
        } else if host_contains("windows") {
            "windows"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "darwin"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            return match host_os {
                Some(host_os) => Err(NdkError::UnsupportedHost(host_os)),
                _ => Err(NdkError::UnsupportedTarget),
            };
        })
    }

    pub fn toolchain_dir(&self) -> Result<PathBuf, NdkError> {
        let arch = Self::host_arch()?;
        let mut toolchain_dir = self
            .ndk_path
            .join("toolchains")
            .join("llvm")
            .join("prebuilt")
            .join(format!("{}-x86_64", arch));
        if !toolchain_dir.exists() {
            toolchain_dir.set_file_name(arch);
        }
        if !toolchain_dir.exists() {
            return Err(NdkError::PathNotFound(toolchain_dir));
        }
        Ok(toolchain_dir)
    }

    pub fn clang(&self) -> Result<(PathBuf, PathBuf), NdkError> {
        let ext = if cfg!(target_os = "windows") {
            "exe"
        } else {
            ""
        };

        let bin_path = self.toolchain_dir()?.join("bin");

        let clang = bin_path.join("clang").with_extension(ext);
        if !clang.exists() {
            return Err(NdkError::PathNotFound(clang));
        }

        let clang_pp = bin_path.join("clang++").with_extension(ext);
        if !clang_pp.exists() {
            return Err(NdkError::PathNotFound(clang_pp));
        }

        Ok((clang, clang_pp))
    }

    pub fn toolchain_bin(&self, name: &str, target: Target) -> Result<PathBuf, NdkError> {
        let ext = if cfg!(target_os = "windows") {
            ".exe"
        } else {
            ""
        };

        let toolchain_path = self.toolchain_dir()?.join("bin");

        // Since r21 (https://github.com/android/ndk/wiki/Changelog-r21) LLVM binutils are included _for testing_;
        // Since r22 (https://github.com/android/ndk/wiki/Changelog-r22) GNU binutils are deprecated in favour of LLVM's;
        // Since r23 (https://github.com/android/ndk/wiki/Changelog-r23) GNU binutils have been removed.
        // To maintain stability with the current ndk-build crate release, prefer GNU binutils for
        // as long as it is provided by the NDK instead of trying to use llvm-* from r21 onwards.
        let gnu_bin = format!("{}-{}{}", target.ndk_triple(), name, ext);
        let gnu_path = toolchain_path.join(&gnu_bin);
        if gnu_path.exists() {
            Ok(gnu_path)
        } else {
            let llvm_bin = format!("llvm-{}{}", name, ext);
            let llvm_path = toolchain_path.join(&llvm_bin);
            llvm_path
                .exists()
                .then(|| llvm_path)
                .ok_or(NdkError::ToolchainBinaryNotFound {
                    toolchain_path,
                    gnu_bin,
                    llvm_bin,
                })
        }
    }

    pub fn prebuilt_dir(&self) -> Result<PathBuf, NdkError> {
        let arch = Self::host_arch()?;
        let prebuilt_dir = self
            .ndk_path
            .join("prebuilt")
            .join(format!("{}-x86_64", arch));
        if !prebuilt_dir.exists() {
            Err(NdkError::PathNotFound(prebuilt_dir))
        } else {
            Ok(prebuilt_dir)
        }
    }

    pub fn ndk_gdb(
        &self,
        launch_dir: impl AsRef<Path>,
        device_serial: Option<&str>,
    ) -> Result<(), NdkError> {
        let abi = self.detect_abi(device_serial)?;
        let jni_dir = launch_dir.as_ref().join("jni");
        std::fs::create_dir_all(&jni_dir)?;
        std::fs::write(
            jni_dir.join("Android.mk"),
            format!("APP_ABI=\"{}\"\nTARGET_OUT=\"\"\n", abi.android_abi()),
        )?;
        let mut ndk_gdb = Command::new(self.prebuilt_dir()?.join("bin").join(cmd!("ndk-gdb")));

        if let Some(device_serial) = &device_serial {
            ndk_gdb.arg("-s").arg(device_serial);
        }

        ndk_gdb
            .arg("--adb")
            .arg(self.adb_path()?)
            .current_dir(launch_dir)
            .status()?;
        Ok(())
    }

    pub fn android_dir(&self) -> Result<PathBuf, NdkError> {
        let android_dir = dirs::home_dir()
            .ok_or_else(|| NdkError::PathNotFound(PathBuf::from("$HOME")))?
            .join(".android");
        std::fs::create_dir_all(&android_dir)?;
        Ok(android_dir)
    }

    pub fn keytool(&self) -> Result<Command, NdkError> {
        if let Ok(keytool) = which::which(bin!("keytool")) {
            return Ok(Command::new(keytool));
        }
        if let Ok(java) = std::env::var("JAVA_HOME") {
            let keytool = PathBuf::from(java).join("bin").join(bin!("keytool"));
            if keytool.exists() {
                return Ok(Command::new(keytool));
            }
        }
        Err(NdkError::CmdNotFound("keytool".to_string()))
    }

    pub fn debug_key(&self) -> Result<Key, NdkError> {
        let path = self.android_dir()?.join("debug.keystore");
        let password = "android".to_string();
        if !path.exists() {
            let mut keytool = self.keytool()?;
            keytool
                .arg("-genkey")
                .arg("-v")
                .arg("-keystore")
                .arg(&path)
                .arg("-storepass")
                .arg(&password)
                .arg("-alias")
                .arg("androiddebugkey")
                .arg("-keypass")
                .arg(&password)
                .arg("-dname")
                .arg("CN=Android Debug,O=Android,C=US")
                .arg("-keyalg")
                .arg("RSA")
                .arg("-keysize")
                .arg("2048")
                .arg("-validity")
                .arg("10000");
            if !keytool.status()?.success() {
                return Err(NdkError::CmdFailed(keytool));
            }
        }
        Ok(Key { path, password })
    }

    pub fn sysroot_lib_dir(&self, target: Target) -> Result<PathBuf, NdkError> {
        let sysroot_lib_dir = self
            .toolchain_dir()?
            .join("sysroot")
            .join("usr")
            .join("lib")
            .join(target.ndk_triple());
        if !sysroot_lib_dir.exists() {
            return Err(NdkError::PathNotFound(sysroot_lib_dir));
        }
        Ok(sysroot_lib_dir)
    }

    pub fn sysroot_platform_lib_dir(
        &self,
        target: Target,
        min_sdk_version: u32,
    ) -> Result<PathBuf, NdkError> {
        let sysroot_lib_dir = self.sysroot_lib_dir(target)?;

        // Look for a platform <= min_sdk_version
        let mut tmp_platform = min_sdk_version;
        while tmp_platform > 1 {
            let path = sysroot_lib_dir.join(tmp_platform.to_string());
            if path.exists() {
                return Ok(path);
            }
            tmp_platform += 1;
        }

        // Look for the minimum API level supported by the NDK
        let mut tmp_platform = min_sdk_version;
        while tmp_platform < 100 {
            let path = sysroot_lib_dir.join(tmp_platform.to_string());
            if path.exists() {
                return Ok(path);
            }
            tmp_platform += 1;
        }

        Err(NdkError::PlatformNotFound(min_sdk_version))
    }

    pub fn detect_abi(&self, device_serial: Option<&str>) -> Result<Target, NdkError> {
        let mut adb = self.adb(device_serial)?;

        let stdout = adb
            .arg("shell")
            .arg("getprop")
            .arg("ro.product.cpu.abi")
            .output()?
            .stdout;
        let abi = std::str::from_utf8(&stdout).or(Err(NdkError::UnsupportedTarget))?;
        Target::from_android_abi(abi.trim())
    }

    pub fn adb(&self, device_serial: Option<&str>) -> Result<Command, NdkError> {
        let mut adb = Command::new(self.adb_path()?);

        if let Some(device_serial) = device_serial {
            adb.arg("-s").arg(device_serial);
        }

        Ok(adb)
    }
}

pub struct Key {
    pub path: PathBuf,
    pub password: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_detect() {
        let ndk = Ndk::from_env().unwrap();
        assert_eq!(ndk.build_tools_version(), "29.0.2");
        assert_eq!(ndk.platforms(), &[29, 28]);
    }
}
