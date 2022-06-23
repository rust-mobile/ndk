macro_rules! bin {
    ($bin:expr) => {
        if cfg!(target_os = "windows") {
            concat!($bin, ".exe")
        } else {
            $bin
        }
    };
}

macro_rules! bat {
    ($bat:expr) => {
        if cfg!(target_os = "windows") {
            concat!($bat, ".bat")
        } else {
            $bat
        }
    };
}

pub mod apk;
pub mod cargo;
pub mod dylibs;
pub mod error;
pub mod manifest;
pub mod ndk;
pub mod readelf;
pub mod target;
