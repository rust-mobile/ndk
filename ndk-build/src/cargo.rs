use crate::error::NdkError;
use crate::ndk::Ndk;
use crate::target::Target;
use std::path::Path;
use std::process::Command;

pub fn cargo_ndk(
    ndk: &Ndk,
    target: Target,
    sdk_version: u32,
    target_dir: impl AsRef<Path>,
) -> Result<Command, NdkError> {
    let triple = target.rust_triple();
    let clang_target = format!("--target={}{}", target.ndk_llvm_triple(), sdk_version);
    let mut cargo = Command::new("cargo");

    const SEP: &str = "\x1f";

    // Read initial CARGO_ENCODED_/RUSTFLAGS
    let mut rustflags = match std::env::var("CARGO_ENCODED_RUSTFLAGS") {
        Ok(val) => {
            if std::env::var_os("RUSTFLAGS").is_some() {
                panic!(
                    "Both `CARGO_ENCODED_RUSTFLAGS` and `RUSTFLAGS` were found in the environment, please clear one or the other before invoking this script"
                );
            }

            val
        }
        Err(std::env::VarError::NotPresent) => {
            match std::env::var("RUSTFLAGS") {
                Ok(val) => {
                    cargo.env_remove("RUSTFLAGS");

                    // Same as cargo
                    // https://github.com/rust-lang/cargo/blob/f6de921a5d807746e972d9d10a4d8e1ca21e1b1f/src/cargo/core/compiler/build_context/target_info.rs#L682-L690
                    val.split(' ')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(SEP)
                }
                Err(std::env::VarError::NotPresent) => String::new(),
                Err(std::env::VarError::NotUnicode(_)) => {
                    panic!("RUSTFLAGS environment variable contains non-unicode characters")
                }
            }
        }
        Err(std::env::VarError::NotUnicode(_)) => {
            panic!("CARGO_ENCODED_RUSTFLAGS environment variable contains non-unicode characters")
        }
    };

    let (clang, clang_pp) = ndk.clang()?;

    // Configure cross-compiler for `cc` crate
    // https://github.com/rust-lang/cc-rs#external-configuration-via-environment-variables
    cargo.env(format!("CC_{}", triple), &clang);
    cargo.env(format!("CFLAGS_{}", triple), &clang_target);
    cargo.env(format!("CXX_{}", triple), &clang_pp);
    cargo.env(format!("CXXFLAGS_{}", triple), &clang_target);

    // Configure LINKER for `rustc`
    // https://doc.rust-lang.org/beta/cargo/reference/environment-variables.html#configuration-environment-variables
    cargo.env(cargo_env_target_cfg("LINKER", triple), &clang);
    if !rustflags.is_empty() {
        rustflags.push_str(SEP);
    }
    rustflags.push_str("-Clink-arg=");
    rustflags.push_str(&clang_target);

    let ar = ndk.toolchain_bin("ar", target)?;
    cargo.env(format!("AR_{}", triple), &ar);
    cargo.env(cargo_env_target_cfg("AR", triple), &ar);

    // Workaround for https://github.com/rust-windowing/android-ndk-rs/issues/149:
    // Rust (1.56 as of writing) still requires libgcc during linking, but this does
    // not ship with the NDK anymore since NDK r23 beta 3.
    // See https://github.com/rust-lang/rust/pull/85806 for a discussion on why libgcc
    // is still required even after replacing it with libunwind in the source.
    // XXX: Add an upper-bound on the Rust version whenever this is not necessary anymore.
    if ndk.build_tag() > 7272597 {
        let cargo_apk_link_dir = target_dir
            .as_ref()
            .join("cargo-apk-temp-extra-link-libraries");
        std::fs::create_dir_all(&cargo_apk_link_dir)
            .map_err(|e| NdkError::IoPathError(cargo_apk_link_dir.clone(), e))?;
        let libgcc = cargo_apk_link_dir.join("libgcc.a");
        std::fs::write(&libgcc, "INPUT(-lunwind)").map_err(|e| NdkError::IoPathError(libgcc, e))?;

        // cdylibs in transitive dependencies still get built and also need this
        // workaround linker flag, yet arguments passed to `cargo rustc` are only
        // forwarded to the final compiler invocation rendering our workaround ineffective.
        // The cargo page documenting this discrepancy (https://doc.rust-lang.org/cargo/commands/cargo-rustc.html)
        // suggests to resort to RUSTFLAGS.
        // Note that `rustflags` will never be empty because of an unconditional `.push_str` above,
        // so we can safely start with appending \x1f here.
        rustflags.push_str(SEP);
        rustflags.push_str("-L");
        rustflags.push_str(SEP);
        rustflags.push_str(
            cargo_apk_link_dir
                .to_str()
                .expect("Target dir must be valid UTF-8"),
        );
    }

    cargo.env("CARGO_ENCODED_RUSTFLAGS", rustflags);

    Ok(cargo)
}

fn cargo_env_target_cfg(tool: &str, target: &str) -> String {
    let utarget = target.replace('-', "_");
    let env = format!("CARGO_TARGET_{}_{}", &utarget, tool);
    env.to_uppercase()
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VersionCode {
    major: u8,
    minor: u8,
    patch: u8,
}

impl VersionCode {
    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn from_semver(version: &str) -> Result<Self, NdkError> {
        let mut iter = version.split(|c1| ['.', '-', '+'].iter().any(|c2| c1 == *c2));
        let mut p = || {
            iter.next()
                .ok_or(NdkError::InvalidSemver)?
                .parse()
                .map_err(|_| NdkError::InvalidSemver)
        };
        Ok(Self::new(p()?, p()?, p()?))
    }

    pub fn to_code(&self, apk_id: u8) -> u32 {
        (apk_id as u32) << 24
            | (self.major as u32) << 16
            | (self.minor as u32) << 8
            | self.patch as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_semver() {
        let v = VersionCode::from_semver("0.0.0").unwrap();
        assert_eq!(v, VersionCode::new(0, 0, 0));
        let v = VersionCode::from_semver("254.254.254-alpha.fix+2").unwrap();
        assert_eq!(v, VersionCode::new(254, 254, 254));
    }
}
