use crate::apk::UnalignedApk;
use crate::error::NdkError;
use crate::target::Target;
use std::collections::HashSet;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::Command;

impl<'a> UnalignedApk<'a> {
    pub fn add_lib_recursively(
        &mut self,
        lib: &Path,
        target: Target,
        search_paths: &[&Path],
    ) -> Result<(), NdkError> {
        let ndk = &self.config().ndk;
        let default_min_sdk = crate::manifest::Sdk::default().min_sdk_version.unwrap();
        let min_sdk_version = self
            .config()
            .manifest
            .sdk
            .min_sdk_version
            .unwrap_or(default_min_sdk);
        let readelf_path = ndk.toolchain_bin("readelf", target)?;

        let android_search_paths = [
            &*ndk.sysroot_lib_dir(target)?,
            &*ndk.sysroot_platform_lib_dir(target, min_sdk_version)?,
        ];

        let mut provided = HashSet::new();
        for path in &android_search_paths {
            for lib in list_libs(path)? {
                if lib != "libc++_shared.so" {
                    provided.insert(lib);
                }
            }
        }

        let mut artifacts = vec![lib.to_path_buf()];
        while let Some(artifact) = artifacts.pop() {
            self.add_lib(&artifact, target)?;
            for need in list_needed_libs(&readelf_path, &artifact)? {
                // c++_shared is available in the NDK but not on-device.
                // Must be bundled with the apk if used:
                // https://developer.android.com/ndk/guides/cpp-support#libc
                let search_paths = if need == "libc++_shared.so" {
                    &android_search_paths
                } else if !provided.contains(&need) {
                    search_paths
                } else {
                    continue;
                };

                if let Some(path) = find_library_path(search_paths, &need)? {
                    if provided.insert(path.file_name().unwrap().to_str().unwrap().to_string()) {
                        artifacts.push(path);
                    }
                } else {
                    eprintln!("Shared library \"{}\" not found.", need);
                }
            }
        }

        Ok(())
    }
}

/// List all linked shared libraries
fn list_needed_libs(readelf_path: &Path, library_path: &Path) -> Result<HashSet<String>, NdkError> {
    let mut readelf = Command::new(readelf_path);
    let output = readelf.arg("-d").arg(library_path).output()?;
    if !output.status.success() {
        return Err(NdkError::CmdFailed(readelf));
    }
    let mut needed = HashSet::new();
    for line in output.stdout.lines() {
        let line = line?;
        if line.contains("(NEEDED)") {
            let lib = line
                .split("Shared library: [")
                .last()
                .and_then(|line| line.split(']').next());
            if let Some(lib) = lib {
                needed.insert(lib.to_string());
            }
        }
    }
    Ok(needed)
}

/// List shared libraries
fn list_libs(path: &Path) -> Result<HashSet<String>, NdkError> {
    let mut libs = HashSet::new();
    let entries = std::fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        if !entry.path().is_dir() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".so") {
                    libs.insert(file_name.to_string());
                }
            }
        }
    }
    Ok(libs)
}

/// Resolves native library using search paths
fn find_library_path<S: AsRef<Path>>(
    paths: &[&Path],
    library: S,
) -> Result<Option<PathBuf>, NdkError> {
    for path in paths {
        let lib_path = path.join(&library);
        if lib_path.exists() {
            return Ok(Some(dunce::canonicalize(lib_path)?));
        }
    }
    Ok(None)
}
