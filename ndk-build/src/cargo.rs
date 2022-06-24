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
    let mut cargo = Command::new("cargo");

    let (clang, clang_pp) = ndk.clang(target, sdk_version)?;
    cargo.env(format!("CC_{}", triple), &clang);
    cargo.env(format!("CXX_{}", triple), &clang_pp);
    cargo.env(cargo_env_target_cfg("LINKER", triple), &clang);

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
        std::fs::create_dir_all(&cargo_apk_link_dir)?;
        std::fs::write(cargo_apk_link_dir.join("libgcc.a"), "INPUT(-lunwind)")
            .expect("Failed to write");

        // cdylibs in transitive dependencies still get built and also need this
        // workaround linker flag, yet arguments passed to `cargo rustc` are only
        // forwarded to the final compiler invocation rendering our workaround ineffective.
        // The cargo page documenting this discrepancy (https://doc.rust-lang.org/cargo/commands/cargo-rustc.html)
        // suggests to resort to RUSTFLAGS, which are updated below:
        let mut rustflags = match std::env::var("CARGO_ENCODED_RUSTFLAGS") {
            Ok(val) => val,
            Err(std::env::VarError::NotPresent) => "".to_string(),
            Err(std::env::VarError::NotUnicode(_)) => {
                panic!("RUSTFLAGS environment variable contains non-unicode characters")
            }
        };
        if !rustflags.is_empty() {
            rustflags.push('\x1f');
        }
        rustflags += "-L\x1f";
        rustflags += cargo_apk_link_dir
            .to_str()
            .expect("Target dir must be valid UTF-8");
        cargo.env("CARGO_ENCODED_RUSTFLAGS", rustflags);
    }

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
