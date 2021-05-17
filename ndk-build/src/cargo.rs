use crate::error::NdkError;
use crate::ndk::Ndk;
use crate::target::Target;
use std::process::Command;

pub fn cargo_ndk(ndk: &Ndk, target: Target, sdk_version: u32) -> Result<Command, NdkError> {
    let triple = target.rust_triple();
    let mut cargo = Command::new("cargo");

    let (clang, clang_pp) = ndk.clang(target, sdk_version)?;
    cargo.env(format!("CC_{}", triple), &clang);
    cargo.env(format!("CXX_{}", triple), &clang_pp);
    cargo.env(cargo_env_target_cfg("LINKER", triple), &clang);

    let ar = ndk.toolchain_bin("ar", target)?;
    cargo.env(format!("AR_{}", triple), &ar);
    cargo.env(cargo_env_target_cfg("AR", triple), &ar);

    Ok(cargo)
}

fn cargo_env_target_cfg(tool: &str, target: &str) -> String {
    let utarget = target.replace("-", "_");
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
