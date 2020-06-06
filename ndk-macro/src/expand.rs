mod backtrace;

#[cfg(feature = "logger")]
mod logger;

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

        let preamble = vec![
            self.expand_backtrace(),
            #[cfg(feature = "logger")]
            self.expand_logger(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

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
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{parse_quote, ItemFn};

    use super::MainAttr;

    fn main(attr: &MainAttr, item: &ItemFn) -> TokenStream {
        attr.expand(item)
    }

    #[test]
    fn main_without_props() {
        let attr = parse_quote! {};
        let item = parse_quote! { fn main() {} };
        let actual = main(&attr, &item);
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
    fn main_with_backtrace_prop() {
        let attr = parse_quote! { backtrace };
        let item = parse_quote! { fn main() {} };
        let actual = main(&attr, &item);
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

    #[cfg(feature = "logger")]
    mod logger {
        use super::*;

        #[test]
        fn main_with_logger_prop_empty() {
            let attr = parse_quote! { logger };
            let item = parse_quote! { fn main() {} };
            let actual = main(&attr, &item);
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
            let attr = parse_quote! { logger(debug) };
            let item = parse_quote! { fn main() {} };
            let actual = main(&attr, &item);
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
            let attr = parse_quote! { logger("my-tag") };
            let item = parse_quote! { fn my_main() {} };
            let actual = main(&attr, &item);
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
            let attr = parse_quote! { logger(warn, "my-tag") };
            let item = parse_quote! { fn my_main() {} };
            let actual = main(&attr, &item);
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
            let attr = parse_quote! { backtrace, logger("my-tag") };
            let item = parse_quote! { fn main() {} };
            let actual = main(&attr, &item);
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
