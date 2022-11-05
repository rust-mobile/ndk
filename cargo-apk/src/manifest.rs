use crate::error::Error;
use ndk_build::apk::StripConfig;
use ndk_build::manifest::AndroidManifest;
use ndk_build::target::Target;
use serde::Deserialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Inheritable<T> {
    Value(T),
    Inherited { workspace: bool },
}

pub(crate) struct Manifest {
    pub(crate) version: Inheritable<String>,
    pub(crate) apk_name: Option<String>,
    pub(crate) android_manifest: AndroidManifest,
    pub(crate) build_targets: Vec<Target>,
    pub(crate) assets: Option<PathBuf>,
    pub(crate) resources: Option<PathBuf>,
    pub(crate) runtime_libs: Option<PathBuf>,
    /// Maps profiles to keystores
    pub(crate) signing: HashMap<String, Signing>,
    pub(crate) reverse_port_forward: HashMap<String, String>,
    pub(crate) strip: StripConfig,
}

impl Manifest {
    pub(crate) fn parse_from_toml(path: &Path) -> Result<Self, Error> {
        let toml = Root::parse_from_toml(path)?;
        // Unlikely to fail as cargo-subcommand should give us a `Cargo.toml` containing
        // a `[package]` table (with a matching `name` when requested by the user)
        let package = toml
            .package
            .unwrap_or_else(|| panic!("Manifest `{:?}` must contain a `[package]`", path));
        let metadata = package
            .metadata
            .unwrap_or_default()
            .android
            .unwrap_or_default();
        Ok(Self {
            version: package.version,
            apk_name: metadata.apk_name,
            android_manifest: metadata.android_manifest,
            build_targets: metadata.build_targets,
            assets: metadata.assets,
            resources: metadata.resources,
            runtime_libs: metadata.runtime_libs,
            signing: metadata.signing,
            reverse_port_forward: metadata.reverse_port_forward,
            strip: metadata.strip,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Root {
    pub(crate) package: Option<Package>,
    pub(crate) workspace: Option<Workspace>,
}

impl Root {
    pub(crate) fn parse_from_toml(path: &Path) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path)?;
        toml::from_str(&contents).map_err(|e| e.into())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Package {
    pub(crate) version: Inheritable<String>,
    pub(crate) metadata: Option<PackageMetadata>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Workspace {
    pub(crate) package: Option<WorkspacePackage>,
}

/// Almost the same as [`Package`], except that this must provide
/// root values instead of possibly inheritable values
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WorkspacePackage {
    pub(crate) version: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct PackageMetadata {
    android: Option<AndroidMetadata>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct AndroidMetadata {
    apk_name: Option<String>,
    #[serde(flatten)]
    android_manifest: AndroidManifest,
    #[serde(default)]
    build_targets: Vec<Target>,
    assets: Option<PathBuf>,
    resources: Option<PathBuf>,
    runtime_libs: Option<PathBuf>,
    /// Maps profiles to keystores
    #[serde(default)]
    signing: HashMap<String, Signing>,
    /// Set up reverse port forwarding before launching the application
    #[serde(default)]
    reverse_port_forward: HashMap<String, String>,
    #[serde(default)]
    strip: StripConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct Signing {
    pub(crate) path: PathBuf,
    pub(crate) keystore_password: String,
}
