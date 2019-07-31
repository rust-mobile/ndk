#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// To allow for bindgen's use of u128 in some cases
#![allow(improper_ctypes)]

include!(concat!(env!("OUT_DIR"), "/ffi.rs"));
