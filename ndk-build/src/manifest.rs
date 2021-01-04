use crate::error::NdkError;
use serde::{Deserialize, Serialize, Serializer};
use std::{fs::File, io::Write, path::Path};

// See https://developer.android.com/guide/topics/manifest/manifest-element
#[serde(rename(serialize = "manifest"))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AndroidManifest {
    #[serde(skip_deserializing)]
    #[serde(rename(serialize = "xmlns:android"))]
    #[serde(default = "default_android_namespace")]
    pub android_namespace: String,
    #[serde(skip_deserializing)]
    #[serde(rename(serialize = "package"))]
    pub package_name: String,
    #[serde(skip_deserializing)]
    #[serde(rename(serialize = "android:versionCode"))]
    pub version_code: u32,
    #[serde(skip_deserializing)]
    #[serde(rename(serialize = "android:versionName"))]
    pub version_name: String,

    #[serde(rename(serialize = "uses-sdk"))]
    #[serde(default)]
    pub sdk: Sdk,
    #[serde(rename(serialize = "uses-feature", deserialize = "feature"))]
    #[serde(default)]
    pub features: Vec<Feature>,
    #[serde(rename(serialize = "uses-permission", deserialize = "permission"))]
    pub permissions: Option<Vec<Permission>>,

    #[serde(default)]
    pub application: Application,
}

impl AndroidManifest {
    pub fn write_to(&self, dir: &Path) -> Result<(), NdkError> {
        let mut file = File::create(dir.join("AndroidManifest.xml"))?;
        let serialized = quick_xml::se::to_string(&self).unwrap();
        writeln!(file, r#"<?xml version="1.0" encoding="utf-8"?>"#)?;
        writeln!(file, "{}", serialized)?;
        Ok(())
    }
}

// See https://developer.android.com/guide/topics/manifest/application-element
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Application {
    #[serde(rename(serialize = "android:debuggable"))]
    pub debuggable: Option<bool>,
    #[serde(rename(serialize = "android:theme"))]
    #[serde(serialize_with = "serialize_fullscreen")]
    pub fullscreen: Option<bool>,
    #[serde(rename(serialize = "android:hasCode"))]
    #[serde(default = "default_has_code")]
    pub has_code: Option<bool>,
    #[serde(rename(serialize = "android:icon"))]
    pub icon: Option<String>,
    #[serde(rename(serialize = "android:label"))]
    #[serde(default)]
    pub label: String,

    #[serde(rename(serialize = "meta-data", deserialize = "metadata"))]
    pub meta_datas: Option<Vec<MetaData>>,
    #[serde(default)]
    pub activity: Activity,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            debuggable: None,
            has_code: default_has_code(),
            icon: None,
            label: "".to_string(),
            fullscreen: None,
            meta_datas: None,
            activity: Activity::default(),
        }
    }
}

fn serialize_fullscreen<S>(v: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match v {
        Some(_) => {
            serializer.serialize_some("@android:style/Theme.DeviceDefault.NoActionBar.Fullscreen")
        }
        None => serializer.serialize_none(),
    }
}

// See https://developer.android.com/guide/topics/manifest/activity-element
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
    #[serde(skip_deserializing)]
    pub name: Option<String>,
    #[serde(rename(serialize = "android:screenOrientation"))]
    pub orientation: Option<String>,

    #[serde(rename(serialize = "meta-data", deserialize = "metadata"))]
    #[serde(default)]
    pub meta_datas: Vec<MetaData>,
    #[serde(rename(serialize = "intent-filter", deserialize = "intent_filter"))]
    #[serde(default)]
    pub intent_filters: Vec<IntentFilter>,
}

impl Default for Activity {
    fn default() -> Self {
        Self {
            name: default_activity_name(),
            label: None,
            orientation: None,
            launch_mode: None,
            config_changes: default_config_changes(),
            meta_datas: Vec::new(),
            intent_filters: Vec::new(),
        }
    }
}

// See https://developer.android.com/guide/topics/manifest/intent-filter-element
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct IntentFilter {
    // Serialize as struct for proper xml format
    #[serde(serialize_with = "serialize_actions")]
    #[serde(rename(serialize = "action"))]
    pub actions: Vec<String>,
    // Serialize as vector of structs for proper xml format
    #[serde(serialize_with = "serialize_catergories")]
    #[serde(rename(serialize = "category"))]
    pub categories: Option<Vec<String>>,
    pub data: Option<Vec<IntentFilterData>>,
}

fn serialize_actions<S>(actions: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeSeq;

    #[derive(Serialize)]
    struct Action {
        #[serde(rename = "android:name")]
        name: String,
    };
    let mut seq = serializer.serialize_seq(Some(actions.len()))?;
    for action in actions {
        seq.serialize_element(&Action {
            name: action.clone(),
        })?;
    }
    seq.end()
}

fn serialize_catergories<S>(
    categories: &Option<Vec<String>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match categories {
        Some(categories) => {
            #[derive(Serialize)]
            struct Category {
                #[serde(rename = "android:name")]
                pub name: String,
            }
            let mut c = Vec::new();
            for category in categories {
                c.push(Category {
                    name: category.clone(),
                });
            }
            serializer.serialize_some(&c)
        }
        None => serializer.serialize_none(),
    }
}

// See https://developer.android.com/guide/topics/manifest/data-element
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

// See https://developer.android.com/guide/topics/manifest/meta-data-element
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MetaData {
    #[serde(rename(serialize = "android:name"))]
    pub name: String,
    #[serde(rename(serialize = "android:value"))]
    pub value: String,
}

// https://developer.android.com/guide/topics/manifest/uses-feature-element
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Feature {
    #[serde(rename(serialize = "android:name"))]
    pub name: Option<String>,
    #[serde(rename(serialize = "android:required"))]
    pub required: Option<bool>,
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

// See https://developer.android.com/guide/topics/manifest/uses-permission-element
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Permission {
    #[serde(rename(serialize = "android:name"))]
    pub name: String,
    #[serde(rename(serialize = "android:maxSdkVersion"))]
    pub max_sdk_version: Option<u32>,
}

// https://developer.android.com/guide/topics/manifest/uses-sdk-element
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

// Manifest namespace attribute should always be set to this value.
//
// https://developer.android.com/guide/topics/manifest/manifest-element
fn default_android_namespace() -> String {
    "http://schemas.android.com/apk/res/android".to_string()
}

fn default_has_code() -> Option<bool> {
    Some(false)
}

fn default_config_changes() -> Option<String> {
    Some("orientation|keyboardHidden|screenSize".to_string())
}

fn default_activity_name() -> Option<String> {
    Some("android.app.NativeActivity".to_string())
}
