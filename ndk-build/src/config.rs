use crate::manifest::{
    ActivityMetadata, ApplicationMetadata, Feature, IntentFilter, IntentFilterData, Permission,
};
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
    pub assets: Option<PathBuf>,
    pub res: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Metadata {
    pub apk_label: Option<String>,
    pub target_sdk_version: Option<u32>,
    pub min_sdk_version: Option<u32>,
    pub icon: Option<String>,
    pub fullscreen: Option<bool>,
    pub orientation: Option<String>,
    pub launch_mode: Option<String>,
    pub opengles_version: Option<(u8, u8)>,
    pub feature: Option<Vec<FeatureConfig>>,
    pub permission: Option<Vec<PermissionConfig>>,
    pub intent_filter: Option<Vec<IntentFilterConfig>>,
    pub application_metadatas: Option<Vec<ApplicationMetadataConfig>>,
    pub activity_metadatas: Option<Vec<ActivityMetadataConfig>>,
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

#[derive(Clone, Debug, Deserialize)]
pub struct IntentFilterConfigData {
    pub scheme: Option<String>,
    pub host: Option<String>,
    pub prefix: Option<String>,
}

impl From<IntentFilterConfigData> for IntentFilterData {
    fn from(config: IntentFilterConfigData) -> Self {
        Self {
            scheme: config.scheme,
            host: config.host,
            prefix: config.prefix,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct IntentFilterConfig {
    name: String,
    data: Vec<IntentFilterConfigData>,
    categories: Vec<String>,
}

impl From<IntentFilterConfig> for IntentFilter {
    fn from(config: IntentFilterConfig) -> Self {
        Self {
            name: config.name,
            data: config
                .data
                .into_iter()
                .map(IntentFilterData::from)
                .rev()
                .collect(),
            categories: config.categories,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApplicationMetadataConfig {
    name: String,
    value: String,
}

impl From<ApplicationMetadataConfig> for ApplicationMetadata {
    fn from(config: ApplicationMetadataConfig) -> Self {
        Self {
            name: config.name,
            value: config.value,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ActivityMetadataConfig {
    name: String,
    value: String,
}

impl From<ActivityMetadataConfig> for ActivityMetadata {
    fn from(config: ActivityMetadataConfig) -> Self {
        Self {
            name: config.name,
            value: config.value,
        }
    }
}
