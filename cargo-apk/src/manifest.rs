use crate::error::Error;
use ndk_build::config::Metadata;
use ndk_build::target::Target;
use serde::Deserialize;
use std::path::Path;

pub struct Manifest {
    pub version: String,
    pub metadata: Metadata,
    pub build_targets: Vec<Target>,
    pub assets: Option<String>,
    pub res: Option<String>,
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
            metadata: metadata.metadata,
            build_targets: metadata.build_targets.unwrap_or_default(),
            assets: metadata.assets,
            res: metadata.res,
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
    metadata: Metadata,
    build_targets: Option<Vec<Target>>,
    assets: Option<String>,
    res: Option<String>,
}
