use crate::error::Error;
use ndk_build::manifest::AndroidManifest;
use ndk_build::target::Target;
use serde::Deserialize;
use std::path::Path;

pub struct Manifest {
    pub version: String,
    pub android_manifest: AndroidManifest,
    pub build_targets: Vec<Target>,
    pub assets: Option<String>,
    pub resources: Option<String>,
}

impl Manifest {
    pub fn parse_from_toml(path: &Path) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path)?;
        let toml: Root = toml::from_str(&contents)?;
        let metadata = toml
            .package
            .metadata
            .unwrap_or_default()
            .android
            .unwrap_or_default();
        Ok(Self {
            version: toml.package.version,
            android_manifest: metadata.android_manifest,
            build_targets: metadata.build_targets,
            assets: metadata.assets,
            resources: metadata.resources,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Root {
    package: Package,
}

#[derive(Debug, Clone, Deserialize)]
struct Package {
    version: String,
    metadata: Option<PackageMetadata>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct PackageMetadata {
    android: Option<AndroidMetadata>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct AndroidMetadata {
    #[serde(flatten)]
    android_manifest: AndroidManifest,
    #[serde(default)]
    build_targets: Vec<Target>,
    assets: Option<String>,
    resources: Option<String>,
}
