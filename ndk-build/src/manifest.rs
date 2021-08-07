use crate::error::NdkError;
use serde::{Deserialize, Serialize, Serializer};
use std::{fs::File, path::Path};

/// Android [manifest element](https://developer.android.com/guide/topics/manifest/manifest-element), containing an [`Application`] element.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "manifest")]
pub struct AndroidManifest {
    #[serde(rename(serialize = "xmlns:android"))]
    #[serde(default = "default_namespace")]
    ns_android: String,
    #[serde(default)]
    pub package: String,
    #[serde(rename(serialize = "android:versionCode"))]
    pub version_code: Option<u32>,
    #[serde(rename(serialize = "android:versionName"))]
    pub version_name: Option<String>,

    #[serde(rename(serialize = "uses-sdk"))]
    #[serde(default)]
    pub sdk: Sdk,

    #[serde(rename(serialize = "uses-feature"))]
    #[serde(default)]
    pub uses_feature: Vec<Feature>,
    #[serde(rename(serialize = "uses-permission"))]
    #[serde(default)]
    pub uses_permission: Vec<Permission>,

    #[serde(default)]
    pub application: Application,
}

impl Default for AndroidManifest {
    fn default() -> Self {
        Self {
            ns_android: default_namespace(),
            package: Default::default(),
            version_code: Default::default(),
            version_name: Default::default(),
            sdk: Default::default(),
            uses_feature: Default::default(),
            uses_permission: Default::default(),
            application: Default::default(),
        }
    }
}

impl AndroidManifest {
    pub fn write_to(&self, dir: &Path) -> Result<(), NdkError> {
        let file = File::create(dir.join("AndroidManifest.xml"))?;
        let w = std::io::BufWriter::new(file);
        quick_xml::se::to_writer(w, &self)?;
        Ok(())
    }
}

/// Android [application element](https://developer.android.com/guide/topics/manifest/application-element), containing an [`Activity`] element.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Application {
    #[serde(rename(serialize = "android:debuggable"))]
    pub debuggable: Option<bool>,
    #[serde(rename(serialize = "android:theme"))]
    pub theme: Option<String>,
    #[serde(rename(serialize = "android:hasCode"))]
    #[serde(default)]
    pub has_code: bool,
    #[serde(rename(serialize = "android:icon"))]
    pub icon: Option<String>,
    #[serde(rename(serialize = "android:label"))]
    #[serde(default)]
    pub label: String,

    #[serde(rename(serialize = "meta-data"))]
    #[serde(default)]
    pub meta_data: Vec<MetaData>,
    #[serde(default)]
    pub activity: Activity,
}

/// Android [activity element](https://developer.android.com/guide/topics/manifest/activity-element).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Activity {
    #[serde(rename(serialize = "android:configChanges"))]
    #[serde(default = "default_config_changes")]
    pub config_changes: Option<String>,
    #[serde(rename(serialize = "android:label"))]
    pub label: Option<String>,
    #[serde(rename(serialize = "android:launchMode"))]
    pub launch_mode: Option<String>,
    #[serde(rename(serialize = "android:name"))]
    #[serde(default = "default_activity_name")]
    pub name: String,
    #[serde(rename(serialize = "android:screenOrientation"))]
    pub orientation: Option<String>,

    #[serde(rename(serialize = "meta-data"))]
    #[serde(default)]
    pub meta_data: Vec<MetaData>,
    /// If no `MAIN` action exists in any intent filter, a default `MAIN` filter is serialized.
    #[serde(serialize_with = "serialize_intents")]
    #[serde(rename(serialize = "intent-filter"))]
    #[serde(default)]
    pub intent_filter: Vec<IntentFilter>,
}

impl Default for Activity {
    fn default() -> Self {
        Self {
            config_changes: default_config_changes(),
            label: None,
            launch_mode: None,
            name: default_activity_name(),
            orientation: None,
            meta_data: Default::default(),
            intent_filter: Default::default(),
        }
    }
}

fn serialize_intents<S>(intent_filters: &[IntentFilter], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeSeq;

    let mut seq = serializer.serialize_seq(None)?;
    for intent_filter in intent_filters {
        seq.serialize_element(intent_filter)?;
    }

    // Check if `intent_filters` contains a `MAIN` action. If not, add a default filter.
    if intent_filters
        .iter()
        .all(|i| i.actions.iter().all(|f| f != "android.intent.action.MAIN"))
    {
        seq.serialize_element(&IntentFilter {
            actions: vec!["android.intent.action.MAIN".to_string()],
            categories: vec!["android.intent.category.LAUNCHER".to_string()],
            data: vec![],
        })?;
    }
    seq.end()
}

/// Android [intent filter element](https://developer.android.com/guide/topics/manifest/intent-filter-element).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct IntentFilter {
    /// Serialize strings wrapped in `<action android:name="..." />`
    #[serde(serialize_with = "serialize_actions")]
    #[serde(rename(serialize = "action"))]
    #[serde(default)]
    pub actions: Vec<String>,
    /// Serialize as vector of structs for proper xml formatting
    #[serde(serialize_with = "serialize_catergories")]
    #[serde(rename(serialize = "category"))]
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub data: Vec<IntentFilterData>,
}

fn serialize_actions<S>(actions: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeSeq;

    #[derive(Serialize)]
    struct Action {
        #[serde(rename = "android:name")]
        name: String,
    }
    let mut seq = serializer.serialize_seq(Some(actions.len()))?;
    for action in actions {
        seq.serialize_element(&Action {
            name: action.clone(),
        })?;
    }
    seq.end()
}

fn serialize_catergories<S>(categories: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeSeq;

    #[derive(Serialize)]
    struct Category {
        #[serde(rename = "android:name")]
        pub name: String,
    }

    let mut seq = serializer.serialize_seq(Some(categories.len()))?;
    for category in categories {
        seq.serialize_element(&Category {
            name: category.clone(),
        })?;
    }
    seq.end()
}

/// Android [intent filter data element](https://developer.android.com/guide/topics/manifest/data-element).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct IntentFilterData {
    #[serde(rename(serialize = "android:scheme"))]
    pub scheme: Option<String>,
    #[serde(rename(serialize = "android:host"))]
    pub host: Option<String>,
    #[serde(rename(serialize = "android:port"))]
    pub port: Option<String>,
    #[serde(rename(serialize = "android:path"))]
    pub path: Option<String>,
    #[serde(rename(serialize = "android:pathPattern"))]
    pub path_pattern: Option<String>,
    #[serde(rename(serialize = "android:pathPrefix"))]
    pub path_prefix: Option<String>,
    #[serde(rename(serialize = "android:mimeType"))]
    pub mime_type: Option<String>,
}

/// Android [meta-data element](https://developer.android.com/guide/topics/manifest/meta-data-element).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MetaData {
    #[serde(rename(serialize = "android:name"))]
    pub name: String,
    #[serde(rename(serialize = "android:value"))]
    pub value: String,
}

/// Android [uses-feature element](https://developer.android.com/guide/topics/manifest/uses-feature-element).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Feature {
    #[serde(rename(serialize = "android:name"))]
    pub name: Option<String>,
    #[serde(rename(serialize = "android:required"))]
    pub required: Option<bool>,
    /// The `version` field is currently used for the following features:
    ///
    /// - `name="android.hardware.vulkan.compute"`: The minimum level of compute features required. See the [Android documentation](https://developer.android.com/reference/android/content/pm/PackageManager#FEATURE_VULKAN_HARDWARE_COMPUTE)
    ///   for available levels and the respective Vulkan features required/provided.
    ///
    /// - `name="android.hardware.vulkan.level"`: The minimum Vulkan requirements. See the [Android documentation](https://developer.android.com/reference/android/content/pm/PackageManager#FEATURE_VULKAN_HARDWARE_LEVEL)
    ///   for available levels and the respective Vulkan features required/provided.
    ///
    /// - `name="android.hardware.vulkan.version"`: Represents the value of Vulkan's `VkPhysicalDeviceProperties::apiVersion`. See the [Android documentation](https://developer.android.com/reference/android/content/pm/PackageManager#FEATURE_VULKAN_HARDWARE_VERSION)
    ///    for available levels and the respective Vulkan features required/provided.
    #[serde(rename(serialize = "android:version"))]
    pub version: Option<u32>,
    #[serde(rename(serialize = "android:glEsVersion"))]
    #[serde(serialize_with = "serialize_opengles_version")]
    pub opengles_version: Option<(u8, u8)>,
}

fn serialize_opengles_version<S>(
    version: &Option<(u8, u8)>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match version {
        Some(version) => {
            let opengles_version = format!("0x{:04}{:04}", version.0, version.1);
            serializer.serialize_some(&opengles_version)
        }
        None => serializer.serialize_none(),
    }
}

/// Android [uses-permission element](https://developer.android.com/guide/topics/manifest/uses-permission-element).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Permission {
    #[serde(rename(serialize = "android:name"))]
    pub name: String,
    #[serde(rename(serialize = "android:maxSdkVersion"))]
    pub max_sdk_version: Option<u32>,
}

/// Android [uses-sdk element](https://developer.android.com/guide/topics/manifest/uses-sdk-element).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Sdk {
    #[serde(rename(serialize = "android:minSdkVersion"))]
    pub min_sdk_version: Option<u32>,
    #[serde(rename(serialize = "android:targetSdkVersion"))]
    pub target_sdk_version: Option<u32>,
    #[serde(rename(serialize = "android:maxSdkVersion"))]
    pub max_sdk_version: Option<u32>,
}

impl Default for Sdk {
    fn default() -> Self {
        Self {
            min_sdk_version: Some(23),
            target_sdk_version: None,
            max_sdk_version: None,
        }
    }
}

fn default_namespace() -> String {
    "http://schemas.android.com/apk/res/android".to_string()
}

fn default_activity_name() -> String {
    "android.app.NativeActivity".to_string()
}

fn default_config_changes() -> Option<String> {
    Some("orientation|keyboardHidden|screenSize".to_string())
}
