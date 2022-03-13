use anyhow::Result;
use ndk_build::ndk::Ndk;
use std::process::Command;

fn main() -> Result<()> {
    let ndk = Ndk::from_env()?;
    let target_platform = ndk.default_target_platform();
    let jar = ndk.android_jar(target_platform)?;
    let tokens = jni_bindgen::jni_bindgen(&jar, Default::default())?;
    std::fs::write("src/lib.rs", tokens.to_string())?;
    Command::new("cargo").arg("fmt").status()?;
    Ok(())
}
