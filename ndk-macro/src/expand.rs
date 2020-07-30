use core::iter::once;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ItemFn;

use crate::{crate_name, parse::MainAttr};

impl MainAttr {
    pub fn expand(&self, main_fn_item: &ItemFn) -> TokenStream {
        let main_fn_name = &main_fn_item.sig.ident;
        let glue_crate = format_ident!(
            "{}",
            crate_name("ndk-glue").expect("No 'ndk-glue' crate found!")
        );

        let preamble = {
            let backtrace = &self.backtrace;
            once(quote! { #backtrace })
        };

        #[cfg(feature = "logger")]
        let preamble = {
            let logger = &self.logger;

            preamble.chain(once(quote! { #logger }))
        };

        quote! {
            #[no_mangle]
            unsafe extern "C" fn ANativeActivity_onCreate(
                activity: *mut std::os::raw::c_void,
                saved_state: *mut std::os::raw::c_void,
                saved_state_size: usize,
            ) {
                #(#preamble)*
                #glue_crate::init(
                    activity as _,
                    saved_state as _,
                    saved_state_size as _,
                    #main_fn_name,
                );
            }

            #main_fn_item
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parse::{BacktraceProp, MainAttr};
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn main_without_props() {
        let attr = MainAttr::default();
        let item = parse_quote! { fn main() {} };
        let actual = attr.expand(&item);
        let expected = quote! {
            #[no_mangle]
            unsafe extern "C" fn ANativeActivity_onCreate(
                activity: *mut std::os::raw::c_void,
                saved_state: *mut std::os::raw::c_void,
                saved_state_size: usize,
            ) {
                ndk_glue::init(
                    activity as _,
                    saved_state as _,
                    saved_state_size as _,
                    main,
                );
            }
            fn main() {}
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn main_with_backtrace_prop_on() {
        let attr = MainAttr {
            backtrace: Some(BacktraceProp::On),
            ..Default::default()
        };
        let item = parse_quote! { fn main() {} };
        let actual = attr.expand(&item);
        let expected = quote! {
            #[no_mangle]
            unsafe extern "C" fn ANativeActivity_onCreate(
                activity: *mut std::os::raw::c_void,
                saved_state: *mut std::os::raw::c_void,
                saved_state_size: usize,
            ) {
                std::env::set_var("RUST_BACKTRACE", "1");
                ndk_glue::init(
                    activity as _,
                    saved_state as _,
                    saved_state_size as _,
                    main,
                );
            }
            fn main() {}
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn main_with_backtrace_prop_full() {
        let attr = MainAttr {
            backtrace: Some(BacktraceProp::Full),
            ..Default::default()
        };
        let item = parse_quote! { fn main() {} };
        let actual = attr.expand(&item);
        let expected = quote! {
            #[no_mangle]
            unsafe extern "C" fn ANativeActivity_onCreate(
                activity: *mut std::os::raw::c_void,
                saved_state: *mut std::os::raw::c_void,
                saved_state_size: usize,
            ) {
                std::env::set_var("RUST_BACKTRACE", "full");
                ndk_glue::init(
                    activity as _,
                    saved_state as _,
                    saved_state_size as _,
                    main,
                );
            }
            fn main() {}
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[cfg(feature = "logger")]
    mod logger {
        use super::*;
        use crate::parse::{LogLevel, LoggerProp};

        #[test]
        fn main_with_logger_prop_empty() {
            let attr = MainAttr {
                logger: Some(LoggerProp::default()),
                ..Default::default()
            };
            let item = parse_quote! { fn main() {} };
            let actual = attr.expand(&item);
            let expected = quote! {
                #[no_mangle]
                unsafe extern "C" fn ANativeActivity_onCreate(
                    activity: *mut std::os::raw::c_void,
                    saved_state: *mut std::os::raw::c_void,
                    saved_state_size: usize,
                ) {
                    android_logger::init_once(
                        android_logger::Config::default()
                    );
                    ndk_glue::init(
                        activity as _,
                        saved_state as _,
                        saved_state_size as _,
                        main,
                    );
                }
                fn main() {}
            };
            assert_eq!(actual.to_string(), expected.to_string());
        }

        #[test]
        fn main_with_logger_prop_with_min_level() {
            let attr = MainAttr {
                logger: Some(LoggerProp {
                    level: Some(LogLevel::Debug),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let item = parse_quote! { fn main() {} };
            let actual = attr.expand(&item);
            let expected = quote! {
                #[no_mangle]
                unsafe extern "C" fn ANativeActivity_onCreate(
                    activity: *mut std::os::raw::c_void,
                    saved_state: *mut std::os::raw::c_void,
                    saved_state_size: usize,
                ) {
                    android_logger::init_once(
                        android_logger::Config::default()
                            .with_min_level(log::Level::Debug)
                    );
                    ndk_glue::init(
                        activity as _,
                        saved_state as _,
                        saved_state_size as _,
                        main,
                    );
                }
                fn main() {}
            };
            assert_eq!(actual.to_string(), expected.to_string());
        }

        #[test]
        fn main_with_logger_prop_with_tag() {
            let attr = MainAttr {
                logger: Some(LoggerProp {
                    tag: Some("my-tag".into()),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let item = parse_quote! { fn my_main() {} };
            let actual = attr.expand(&item);
            let expected = quote! {
                #[no_mangle]
                unsafe extern "C" fn ANativeActivity_onCreate(
                    activity: *mut std::os::raw::c_void,
                    saved_state: *mut std::os::raw::c_void,
                    saved_state_size: usize,
                ) {
                    android_logger::init_once(
                        android_logger::Config::default()
                            .with_tag("my-tag")
                    );
                    ndk_glue::init(
                        activity as _,
                        saved_state as _,
                        saved_state_size as _,
                        my_main,
                    );
                }
                fn my_main() {}
            };
            assert_eq!(actual.to_string(), expected.to_string());
        }

        #[test]
        fn main_with_logger_prop_with_min_level_and_with_tag() {
            let attr = MainAttr {
                logger: Some(LoggerProp {
                    level: Some(LogLevel::Warn),
                    tag: Some("my-tag".into()),
                }),
                ..Default::default()
            };
            let item = parse_quote! { fn my_main() {} };
            let actual = attr.expand(&item);
            let expected = quote! {
                #[no_mangle]
                unsafe extern "C" fn ANativeActivity_onCreate(
                    activity: *mut std::os::raw::c_void,
                    saved_state: *mut std::os::raw::c_void,
                    saved_state_size: usize,
                ) {
                    android_logger::init_once(
                        android_logger::Config::default()
                            .with_tag("my-tag")
                            .with_min_level(log::Level::Warn)
                    );
                    ndk_glue::init(
                        activity as _,
                        saved_state as _,
                        saved_state_size as _,
                        my_main,
                    );
                }
                fn my_main() {}
            };
            assert_eq!(actual.to_string(), expected.to_string());
        }

        #[test]
        fn main_with_backtrace_prop_and_logger_prop() {
            let attr = MainAttr {
                backtrace: Some(BacktraceProp::On),
                logger: Some(LoggerProp {
                    tag: Some("my-tag".into()),
                    ..Default::default()
                }),
            };
            let item = parse_quote! { fn main() {} };
            let actual = attr.expand(&item);
            let expected = quote! {
                #[no_mangle]
                unsafe extern "C" fn ANativeActivity_onCreate(
                    activity: *mut std::os::raw::c_void,
                    saved_state: *mut std::os::raw::c_void,
                    saved_state_size: usize,
                ) {
                    std::env::set_var("RUST_BACKTRACE", "1");
                    android_logger::init_once(
                        android_logger::Config::default()
                            .with_tag("my-tag")
                    );
                    ndk_glue::init(
                        activity as _,
                        saved_state as _,
                        saved_state_size as _,
                        main,
                    );
                }
                fn main() {}
            };
            assert_eq!(actual.to_string(), expected.to_string());
        }
    }
}
