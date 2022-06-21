use crate::error::Error;
use crate::manifest::Manifest;
use cargo_subcommand::{Artifact, CrateType, Profile, Subcommand};
use ndk_build::apk::{Apk, ApkConfig};
use ndk_build::cargo::{cargo_ndk, VersionCode};
use ndk_build::dylibs::get_libs_search_paths;
use ndk_build::error::NdkError;
use ndk_build::manifest::{IntentFilter, MetaData};
use ndk_build::ndk::{Key, Ndk};
use ndk_build::target::Target;
use std::path::PathBuf;
use std::process::Command;

pub struct ApkBuilder<'a> {
    cmd: &'a Subcommand,
    ndk: Ndk,
    manifest: Manifest,
    build_dir: PathBuf,
    build_targets: Vec<Target>,
}

impl<'a> ApkBuilder<'a> {
    pub fn from_subcommand(cmd: &'a Subcommand) -> Result<Self, Error> {
        let ndk = Ndk::from_env()?;
        let mut manifest = Manifest::parse_from_toml(cmd.manifest())?;
        let build_targets = if let Some(target) = cmd.target() {
            vec![Target::from_rust_triple(target)?]
        } else if !manifest.build_targets.is_empty() {
            manifest.build_targets.clone()
        } else {
            vec![ndk.detect_abi().unwrap_or(Target::Arm64V8a)]
        };
        let build_dir = dunce::simplified(cmd.target_dir())
            .join(cmd.profile())
            .join("apk");

        // Set default Android manifest values
        if manifest
            .android_manifest
            .version_name
            .replace(manifest.version.clone())
            .is_some()
        {
            panic!("version_name should not be set in TOML");
        }

        if manifest
            .android_manifest
            .version_code
            .replace(VersionCode::from_semver(&manifest.version)?.to_code(1))
            .is_some()
        {
            panic!("version_code should not be set in TOML");
        }

        let target_sdk_version = *manifest
            .android_manifest
            .sdk
            .target_sdk_version
            .get_or_insert_with(|| ndk.default_target_platform());

        manifest
            .android_manifest
            .application
            .debuggable
            .get_or_insert_with(|| *cmd.profile() == Profile::Dev);

        let activity = &mut manifest.android_manifest.application.activity;

        // Add a default `MAIN` action to launch the activity, if the user didn't supply it by hand.
        if activity
            .intent_filter
            .iter()
            .all(|i| i.actions.iter().all(|f| f != "android.intent.action.MAIN"))
        {
            activity.intent_filter.push(IntentFilter {
                actions: vec!["android.intent.action.MAIN".to_string()],
                categories: vec!["android.intent.category.LAUNCHER".to_string()],
                data: vec![],
            });
        }

        // Export the sole Rust activity on Android S and up, if the user didn't explicitly do so.
        // Without this, apps won't start on S+.
        // https://developer.android.com/about/versions/12/behavior-changes-12#exported
        if target_sdk_version >= 31 {
            activity.exported.get_or_insert(true);
        }

        Ok(Self {
            cmd,
            ndk,
            manifest,
            build_dir,
            build_targets,
        })
    }

    pub fn check(&self) -> Result<(), Error> {
        for target in &self.build_targets {
            let mut cargo = cargo_ndk(
                &self.ndk,
                *target,
                self.min_sdk_version(),
                self.cmd.target_dir(),
            )?;
            cargo.arg("check");
            if self.cmd.target().is_none() {
                let triple = target.rust_triple();
                cargo.arg("--target").arg(triple);
            }
            cargo.args(self.cmd.args());
            if !cargo.status()?.success() {
                return Err(NdkError::CmdFailed(cargo).into());
            }
        }
        Ok(())
    }

    pub fn build(&self, artifact: &Artifact) -> Result<Apk, Error> {
        // Set artifact specific manifest default values.
        let mut manifest = self.manifest.android_manifest.clone();

        if manifest.package.is_empty() {
            manifest.package = match artifact {
                Artifact::Root(name) => format!("rust.{}", name.replace('-', "_")),
                Artifact::Example(name) => format!("rust.example.{}", name.replace('-', "_")),
            };
        }

        if manifest.application.label.is_empty() {
            manifest.application.label = artifact.name().to_string();
        }

        manifest.application.activity.meta_data.push(MetaData {
            name: "android.app.lib_name".to_string(),
            value: artifact.name().replace('-', "_"),
        });

        let crate_path = self.cmd.manifest().parent().expect("invalid manifest path");

        let is_debug_profile = *self.cmd.profile() == Profile::Dev;

        let assets = self
            .manifest
            .assets
            .as_ref()
            .map(|assets| dunce::simplified(&crate_path.join(&assets)).to_owned());
        let resources = self
            .manifest
            .resources
            .as_ref()
            .map(|res| dunce::simplified(&crate_path.join(&res)).to_owned());
        let runtime_libs = self
            .manifest
            .runtime_libs
            .as_ref()
            .map(|libs| dunce::simplified(&crate_path.join(&libs)).to_owned());
        let apk_name = self
            .manifest
            .apk_name
            .clone()
            .unwrap_or_else(|| artifact.name().to_string());

        let config = ApkConfig {
            ndk: self.ndk.clone(),
            build_dir: self.build_dir.join(artifact),
            apk_name,
            assets,
            resources,
            manifest,
            disable_aapt_compression: is_debug_profile,
        };
        let apk = config.create_apk()?;

        for target in &self.build_targets {
            let triple = target.rust_triple();
            let build_dir = dunce::simplified(self.cmd.target_dir())
                .join(triple)
                .join(self.cmd.profile());
            let artifact = build_dir
                .join(artifact)
                .join(artifact.file_name(CrateType::Cdylib, triple));

            let mut cargo = cargo_ndk(
                &self.ndk,
                *target,
                self.min_sdk_version(),
                self.cmd.target_dir(),
            )?;
            cargo.arg("build");
            if self.cmd.target().is_none() {
                cargo.arg("--target").arg(triple);
            }
            cargo.args(self.cmd.args());

            if !cargo.status()?.success() {
                return Err(NdkError::CmdFailed(cargo).into());
            }

            let mut libs_search_paths =
                get_libs_search_paths(self.cmd.target_dir(), triple, self.cmd.profile().as_ref())?;
            libs_search_paths.push(build_dir.join("deps"));

            let libs_search_paths = libs_search_paths
                .iter()
                .map(|path| path.as_path())
                .collect::<Vec<_>>();

            apk.add_lib_recursively(&artifact, *target, libs_search_paths.as_slice())?;

            if let Some(runtime_libs) = &runtime_libs {
                apk.add_runtime_libs(runtime_libs, *target, libs_search_paths.as_slice())?;
            }
        }

        let profile_name = match self.cmd.profile() {
            Profile::Dev => "dev",
            Profile::Release => "release",
            Profile::Custom(c) => c.as_str(),
        };

        let signing_key = self.manifest.signing.get(profile_name);

        let signing_key = match (signing_key, is_debug_profile) {
            (Some(signing), _) => Key {
                path: crate_path.join(&signing.path),
                password: signing.keystore_password.clone(),
            },
            (None, true) => self.ndk.debug_key()?,
            (None, false) => return Err(Error::MissingReleaseKey(profile_name.to_owned())),
        };

        Ok(apk.align()?.sign(signing_key)?)
    }

    pub fn run(&self, artifact: &Artifact) -> Result<(), Error> {
        let apk = self.build(artifact)?;
        apk.install()?;
        apk.start()?;
        Ok(())
    }

    pub fn gdb(&self, artifact: &Artifact) -> Result<(), Error> {
        self.run(artifact)?;
        let abi = self.ndk.detect_abi()?;
        let target_dir = self.build_dir.join(artifact);
        let jni_dir = target_dir.join("jni");
        std::fs::create_dir_all(&jni_dir)?;
        std::fs::write(
            jni_dir.join("Android.mk"),
            format!("APP_ABI=\"{}\"\nTARGET_OUT=\"\"\n", abi.android_abi()),
        )?;
        Command::new(self.ndk.ndk().join("ndk-gdb"))
            .current_dir(target_dir)
            .status()?;
        Ok(())
    }

    pub fn default(&self) -> Result<(), Error> {
        for target in &self.build_targets {
            let mut cargo = cargo_ndk(
                &self.ndk,
                *target,
                self.min_sdk_version(),
                self.cmd.target_dir(),
            )?;
            cargo.args(self.cmd.args());

            if self.cmd.target().is_none() {
                let triple = target.rust_triple();
                cargo.arg("--target").arg(triple);
            }

            if !cargo.status()?.success() {
                return Err(NdkError::CmdFailed(cargo).into());
            }
        }
        Ok(())
    }

    /// Returns `minSdkVersion` for use in compiler target selection:
    /// <https://developer.android.com/ndk/guides/sdk-versions#minsdkversion>
    ///
    /// Has a lower bound of `23` to retain backwards compatibility with
    /// the previous default.
    fn min_sdk_version(&self) -> u32 {
        self.manifest
            .android_manifest
            .sdk
            .min_sdk_version
            .unwrap_or(23)
            .max(23)
    }
}
