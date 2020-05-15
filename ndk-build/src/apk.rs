use crate::config::{Config, Metadata};
use crate::error::NdkError;
use crate::manifest::Manifest;
use crate::ndk::{Key, Ndk};
use crate::target::Target;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct ApkConfig {
    pub ndk: Ndk,
    pub build_dir: PathBuf,
    pub assets: Option<String>,
    pub res: Option<String>,
    pub manifest: Manifest,
}

impl ApkConfig {
    pub fn from_config(config: Config, metadata: Metadata) -> Self {
        let target_sdk_version = metadata
            .target_sdk_version
            .unwrap_or_else(|| config.ndk.default_platform());
        let features = metadata
            .feature
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect();
        let permissions = metadata
            .permission
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect();
        let application_metadatas = metadata
            .application_metadatas
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect();

        let manifest = Manifest {
            package_name: config.package_name,
            package_label: config.package_label,
            version_name: config.version_name,
            version_code: config.version_code,
            split: config.split,
            target_name: config.target_name,
            debuggable: config.debuggable,
            target_sdk_version,
            min_sdk_version: metadata.min_sdk_version.unwrap_or(23),
            opengles_version: metadata.opengles_version.unwrap_or((3, 1)),
            features,
            permissions,
            icon: metadata.icon,
            fullscreen: metadata.fullscreen.unwrap_or(false),
            application_metadatas,
        };
        Self {
            ndk: config.ndk,
            build_dir: config.build_dir,
            assets: config.assets,
            res: config.res,
            manifest,
        }
    }

    fn build_tool(&self, tool: &'static str) -> Result<Command, NdkError> {
        let mut cmd = self.ndk.build_tool(tool)?;
        cmd.current_dir(&self.build_dir);
        Ok(cmd)
    }

    fn unaligned_apk(&self) -> PathBuf {
        self.build_dir
            .join(format!("{}-unaligned.apk", self.manifest.package_label))
    }

    fn apk(&self) -> PathBuf {
        self.build_dir
            .join(format!("{}.apk", self.manifest.package_label))
    }

    pub fn create_apk(&self) -> Result<UnalignedApk, NdkError> {
        std::fs::create_dir_all(&self.build_dir)?;
        self.manifest.write_to(&self.build_dir)?;

        let mut aapt = self.build_tool(bin!("aapt"))?;
        aapt.arg("package")
            .arg("-f")
            .arg("-F")
            .arg(self.unaligned_apk())
            .arg("-M")
            .arg("AndroidManifest.xml")
            .arg("-I")
            .arg(self.ndk.android_jar(self.manifest.target_sdk_version)?);

        if let Some(res) = &self.res {
            aapt.arg("-S").arg(res);
        }

        if let Some(assets) = &self.assets {
            aapt.arg("-A").arg(assets);
        }

        if !aapt.status()?.success() {
            return Err(NdkError::CmdFailed(aapt));
        }

        Ok(UnalignedApk(self))
    }
}

pub struct UnalignedApk<'a>(&'a ApkConfig);

impl<'a> UnalignedApk<'a> {
    pub fn config(&self) -> &ApkConfig {
        self.0
    }

    pub fn add_lib(&self, path: &Path, target: Target) -> Result<(), NdkError> {
        if !path.exists() {
            return Err(NdkError::PathNotFound(path.into()));
        }
        let abi = target.android_abi();
        let file_name = path.file_name().unwrap();
        let out = self.0.build_dir.join("lib").join(abi);
        std::fs::create_dir_all(&out)?;
        std::fs::copy(path, out.join(&file_name))?;

        let mut aapt = self.0.build_tool(bin!("aapt"))?;
        aapt.arg("add").arg(self.0.unaligned_apk()).arg(format!(
            "lib/{}/{}",
            abi,
            file_name.to_str().unwrap()
        ));
        if !aapt.status()?.success() {
            return Err(NdkError::CmdFailed(aapt));
        }
        Ok(())
    }

    pub fn align(self) -> Result<UnsignedApk<'a>, NdkError> {
        let mut zipalign = self.0.build_tool(bin!("zipalign"))?;
        zipalign
            .arg("-f")
            .arg("-v")
            .arg("4")
            .arg(self.0.unaligned_apk())
            .arg(self.0.apk());
        if !zipalign.status()?.success() {
            return Err(NdkError::CmdFailed(zipalign));
        }
        Ok(UnsignedApk(self.0))
    }
}

pub struct UnsignedApk<'a>(&'a ApkConfig);

impl<'a> UnsignedApk<'a> {
    pub fn sign(self, key: Key) -> Result<Apk, NdkError> {
        let mut apksigner = self.0.build_tool(bat!("apksigner"))?;
        apksigner
            .arg("sign")
            .arg("--ks")
            .arg(&key.path)
            .arg("--ks-pass")
            .arg(format!("pass:{}", &key.password))
            .arg(self.0.apk());
        if !apksigner.status()?.success() {
            return Err(NdkError::CmdFailed(apksigner));
        }
        Ok(Apk::from_config(self.0))
    }
}

pub struct Apk {
    path: PathBuf,
    package_name: String,
    ndk: Ndk,
}

impl Apk {
    pub fn from_config(config: &ApkConfig) -> Self {
        let ndk = config.ndk.clone();
        Self {
            path: config.apk(),
            package_name: config.manifest.package_name.clone(),
            ndk,
        }
    }

    pub fn install(&self) -> Result<(), NdkError> {
        let mut adb = self.ndk.platform_tool(bin!("adb"))?;
        adb.arg("install").arg("-r").arg(&self.path);
        if !adb.status()?.success() {
            return Err(NdkError::CmdFailed(adb));
        }
        Ok(())
    }

    pub fn start(&self) -> Result<(), NdkError> {
        let mut adb = self.ndk.platform_tool(bin!("adb"))?;
        adb.arg("shell")
            .arg("am")
            .arg("start")
            .arg("-a")
            .arg("android.intent.action.MAIN")
            .arg("-n")
            .arg(format!("{}/android.app.NativeActivity", &self.package_name));
        if !adb.status()?.success() {
            return Err(NdkError::CmdFailed(adb));
        }
        Ok(())
    }
}
