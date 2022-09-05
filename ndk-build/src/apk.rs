use crate::error::NdkError;
use crate::manifest::AndroidManifest;
use crate::ndk::{Key, Ndk};
use crate::target::Target;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct ApkConfig {
    pub ndk: Ndk,
    pub build_dir: PathBuf,
    pub apk_name: String,
    pub assets: Option<PathBuf>,
    pub resources: Option<PathBuf>,
    pub manifest: AndroidManifest,
    pub disable_aapt_compression: bool,
}

impl ApkConfig {
    fn build_tool(&self, tool: &'static str) -> Result<Command, NdkError> {
        let mut cmd = self.ndk.build_tool(tool)?;
        cmd.current_dir(&self.build_dir);
        Ok(cmd)
    }

    fn unaligned_apk(&self) -> PathBuf {
        self.build_dir
            .join(format!("{}-unaligned.apk", self.apk_name))
    }

    fn apk(&self) -> PathBuf {
        self.build_dir.join(format!("{}.apk", self.apk_name))
    }

    pub fn create_apk(&self) -> Result<UnalignedApk, NdkError> {
        std::fs::create_dir_all(&self.build_dir)?;
        self.manifest.write_to(&self.build_dir)?;

        let target_sdk_version = self
            .manifest
            .sdk
            .target_sdk_version
            .unwrap_or_else(|| self.ndk.default_target_platform());
        let mut aapt = self.build_tool(bin!("aapt"))?;
        aapt.arg("package")
            .arg("-f")
            .arg("-F")
            .arg(self.unaligned_apk())
            .arg("-M")
            .arg("AndroidManifest.xml")
            .arg("-I")
            .arg(self.ndk.android_jar(target_sdk_version)?);

        if self.disable_aapt_compression {
            aapt.arg("-0").arg("");
        }

        if let Some(res) = &self.resources {
            aapt.arg("-S").arg(res);
        }

        if let Some(assets) = &self.assets {
            aapt.arg("-A").arg(assets);
        }

        if !aapt.status()?.success() {
            return Err(NdkError::CmdFailed(aapt));
        }

        Ok(UnalignedApk {
            config: self,
            pending_libs: HashSet::default(),
        })
    }
}

pub struct UnalignedApk<'a> {
    config: &'a ApkConfig,
    pending_libs: HashSet<String>,
}

impl<'a> UnalignedApk<'a> {
    pub fn config(&self) -> &ApkConfig {
        self.config
    }

    pub fn add_lib(&mut self, path: &Path, target: Target) -> Result<(), NdkError> {
        if !path.exists() {
            return Err(NdkError::PathNotFound(path.into()));
        }
        let abi = target.android_abi();
        let lib_path = Path::new("lib").join(abi).join(path.file_name().unwrap());
        let out = self.config.build_dir.join(&lib_path);
        std::fs::create_dir_all(out.parent().unwrap())?;
        std::fs::copy(path, out)?;

        // Pass UNIX path separators to `aapt` on non-UNIX systems, ensuring the resulting separator
        // is compatible with the target device instead of the host platform.
        // Otherwise, it results in a runtime error when loading the NativeActivity `.so` library.
        let lib_path_unix = lib_path.to_str().unwrap().replace('\\', "/");

        self.pending_libs.insert(lib_path_unix);

        Ok(())
    }

    pub fn add_runtime_libs(
        &mut self,
        path: &Path,
        target: Target,
        search_paths: &[&Path],
    ) -> Result<(), NdkError> {
        let abi_dir = path.join(target.android_abi());
        for entry in fs::read_dir(&abi_dir).map_err(|e| NdkError::IoPathError(abi_dir, e))? {
            let entry = entry?;
            let path = entry.path();
            if path.extension() == Some(OsStr::new("so")) {
                self.add_lib_recursively(&path, target, search_paths)?;
            }
        }
        Ok(())
    }

    pub fn add_pending_libs_and_align(self) -> Result<UnsignedApk<'a>, NdkError> {
        let mut aapt = self.config.build_tool(bin!("aapt"))?;
        aapt.arg("add");

        if self.config.disable_aapt_compression {
            aapt.arg("-0").arg("");
        }

        aapt.arg(self.config.unaligned_apk());

        for lib_path_unix in self.pending_libs {
            aapt.arg(lib_path_unix);
        }

        if !aapt.status()?.success() {
            return Err(NdkError::CmdFailed(aapt));
        }

        let mut zipalign = self.config.build_tool(bin!("zipalign"))?;
        zipalign
            .arg("-f")
            .arg("-v")
            .arg("4")
            .arg(self.config.unaligned_apk())
            .arg(self.config.apk());
        if !zipalign.status()?.success() {
            return Err(NdkError::CmdFailed(zipalign));
        }
        Ok(UnsignedApk(self.config))
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
            package_name: config.manifest.package.clone(),
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
