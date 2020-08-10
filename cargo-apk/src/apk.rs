use crate::error::Error;
use crate::manifest::Manifest;
use cargo_subcommand::{Artifact, CrateType, Profile, Subcommand};
use ndk_build::apk::{Apk, ApkConfig};
use ndk_build::cargo::{cargo_apk, VersionCode};
use ndk_build::config::Config;
use ndk_build::error::NdkError;
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
        let manifest = Manifest::parse_from_toml(cmd.manifest())?;
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
        Ok(Self {
            cmd,
            ndk,
            manifest,
            build_dir,
            build_targets,
        })
    }

    pub fn build(&self, artifact: &Artifact) -> Result<Apk, Error> {
        let package_name = match artifact {
            Artifact::Root(name) => format!("rust.{}", name.replace("-", "_")),
            Artifact::Example(name) => format!("rust.example.{}", name.replace("-", "_")),
        };
        let config = Config {
            ndk: self.ndk.clone(),
            build_dir: self.build_dir.join(artifact),
            package_name,
            package_label: artifact.name().to_string(),
            version_name: self.manifest.version.clone(),
            version_code: VersionCode::from_semver(&self.manifest.version)?.to_code(1),
            split: None,
            target_name: artifact.name().replace("-", "_"),
            debuggable: *self.cmd.profile() == Profile::Dev,
            assets: self.manifest.assets.as_ref().map(|assets| {
                self.cmd
                    .manifest()
                    .parent()
                    .expect("invalid manifest path")
                    .join(&assets)
            }),
            res: self.manifest.res.clone(),
        };
        let config = ApkConfig::from_config(config, self.manifest.metadata.clone());
        let apk = config.create_apk()?;

        for target in &self.build_targets {
            let triple = target.rust_triple();
            let build_dir = dunce::simplified(self.cmd.target_dir())
                .join(target.rust_triple())
                .join(self.cmd.profile());
            let artifact = build_dir
                .join(artifact)
                .join(artifact.file_name(CrateType::Cdylib, &triple));

            let target_sdk_version = config.manifest.target_sdk_version;

            let mut cargo = cargo_apk(&config.ndk, *target, target_sdk_version)?;
            cargo.arg("build");
            if self.cmd.target().is_none() {
                cargo.arg("--target").arg(target.rust_triple());
            }
            cargo.args(self.cmd.args());
            if !cargo.status()?.success() {
                return Err(NdkError::CmdFailed(cargo).into());
            }

            apk.add_lib_recursively(&artifact, *target, &[&build_dir.join("deps")])?;
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
        let target_sdk_version = self
            .manifest
            .metadata
            .target_sdk_version
            .unwrap_or_else(|| ndk.default_platform());
        for target in &self.build_targets {
            let mut cargo = cargo_apk(&ndk, *target, target_sdk_version)?;
            cargo.args(self.cmd.args());
            if !cargo.status()?.success() {
                return Err(NdkError::CmdFailed(cargo).into());
            }
        }
        Ok(())
    }
}
