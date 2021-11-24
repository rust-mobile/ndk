use crate::error::Error;
use crate::manifest::Manifest;
use cargo_subcommand::{Artifact, CrateType, Profile, Subcommand};
use ndk_build::apk::{Apk, ApkConfig};
use ndk_build::cargo::{cargo_ndk, VersionCode};
use ndk_build::dylibs::get_libs_search_paths;
use ndk_build::error::NdkError;
use ndk_build::manifest::MetaData;
use ndk_build::ndk::Ndk;
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
        assert!(manifest.android_manifest.package.is_empty());

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

        manifest
            .android_manifest
            .sdk
            .target_sdk_version
            .get_or_insert(ndk.default_platform());

        manifest
            .android_manifest
            .application
            .debuggable
            .get_or_insert(*cmd.profile() == Profile::Dev);

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
            let triple = target.rust_triple();
            let mut cargo = cargo_ndk(&self.ndk, *target, self.min_sdk_version())?;
            cargo.arg("check");
            if self.cmd.target().is_none() {
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
        let package_name = match artifact {
            Artifact::Root(name) => format!("rust.{}", name.replace("-", "_")),
            Artifact::Example(name) => format!("rust.example.{}", name.replace("-", "_")),
        };

        // Set artifact specific manifest default values.
        let mut manifest = self.manifest.android_manifest.clone();
        manifest.package = package_name;
        if manifest.application.label.is_empty() {
            manifest.application.label = artifact.name().to_string();
        }

        manifest.application.activity.meta_data.push(MetaData {
            name: "android.app.lib_name".to_string(),
            value: artifact.name().replace("-", "_"),
        });

        let crate_path = self.cmd.manifest().parent().expect("invalid manifest path");

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

            let mut cargo = cargo_ndk(&config.ndk, *target, self.min_sdk_version())?;
            cargo.arg("rustc");
            if self.cmd.target().is_none() {
                cargo.arg("--target").arg(triple);
            }
            cargo.args(self.cmd.args());

            // Workaround for https://github.com/rust-windowing/android-ndk-rs/issues/149:
            // Rust (1.56 as of writing) still requires libgcc during linking, but this does
            // not ship with the NDK anymore since NDK r23 beta 3.
            // See https://github.com/rust-lang/rust/pull/85806 for a discussion on why libgcc
            // is still required even after replacing it with libunwind in the source.
            // XXX: Add an upper-bound on the Rust version whenever this is not necessary anymore.
            if self.ndk.build_tag() > 7272597 {
                if !self.cmd.args().contains(&"--".to_owned()) {
                    cargo.arg("--");
                }
                let cargo_apk_link_dir = self
                    .cmd
                    .target_dir()
                    .join("cargo-apk-temp-extra-link-libraries");
                std::fs::create_dir_all(&cargo_apk_link_dir)?;
                std::fs::write(cargo_apk_link_dir.join("libgcc.a"), "INPUT(-lunwind)")
                    .expect("Failed to write");
                cargo.arg("-L").arg(cargo_apk_link_dir);
            }

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

        Ok(apk.align()?.sign(config.ndk.debug_key()?)?)
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
        Command::new("ndk-gdb").current_dir(target_dir).status()?;
        Ok(())
    }

    pub fn default(&self) -> Result<(), Error> {
        let ndk = Ndk::from_env()?;
        for target in &self.build_targets {
            let mut cargo = cargo_ndk(&ndk, *target, self.min_sdk_version())?;
            cargo.args(self.cmd.args());
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
