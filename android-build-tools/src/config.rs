use crate::manifest::{Feature, Permission};
use crate::ndk::Ndk;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub ndk: Ndk,
    pub build_dir: PathBuf,
    pub package_name: String,
    pub package_label: String,
    pub version_name: String,
    pub version_code: u32,
    pub split: Option<String>,
    pub target_name: String,
    pub debuggable: bool,
    pub assets: Option<String>,
    pub res: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Metadata {
    pub target_sdk_version: Option<u32>,
    pub min_sdk_version: Option<u32>,
    pub icon: Option<String>,
    pub fullscreen: Option<bool>,
    pub opengles_version: Option<(u8, u8)>,
    pub feature: Option<Vec<FeatureConfig>>,
    pub permission: Option<Vec<PermissionConfig>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FeatureConfig {
    name: String,
    required: Option<bool>,
}

impl From<FeatureConfig> for Feature {
    fn from(config: FeatureConfig) -> Self {
        Self {
            name: config.name,
            required: config.required.unwrap_or(true),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PermissionConfig {
    name: String,
    max_sdk_version: Option<u32>,
}

impl From<PermissionConfig> for Permission {
    fn from(config: PermissionConfig) -> Self {
        Self {
            name: config.name,
            max_sdk_version: config.max_sdk_version,
        }
    }
}
