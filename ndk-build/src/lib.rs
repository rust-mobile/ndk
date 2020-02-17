macro_rules! bin {
    ($bin:expr) => {{
        #[cfg(not(target_os = "windows"))]
        let bin = $bin;
        #[cfg(target_os = "windows")]
        let bin = format!("{}.exe", $bin);
        bin
    }};
}

macro_rules! bat {
    ($bat:expr) => {{
        #[cfg(not(target_os = "windows"))]
        let bat = $bat;
        #[cfg(target_os = "windows")]
        let bat = format!("{}.bat", $bat);
        bat
    }};
}

pub mod apk;
pub mod cargo;
pub mod config;
pub mod error;
pub mod manifest;
pub mod ndk;
pub mod readelf;
pub mod target;
