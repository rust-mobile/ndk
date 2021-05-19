use core::iter::once;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::ItemFn;

use crate::{
    helper::crate_path,
    parse::{BacktraceProp, MainAttr},
};

impl ToTokens for BacktraceProp {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use BacktraceProp::*;

        let prop = match self {
            On => Some(quote! { "1" }),
            Full => Some(quote! { "full" }),
        };

        tokens.extend(quote! {
            std::env::set_var("RUST_BACKTRACE", #prop);
        });
    }
}

#[cfg(feature = "logger")]
mod logger {
    use super::*;
    use crate::parse::{LogLevel, LoggerProp};
    use syn::Path;

    impl LoggerProp {
        pub(crate) fn expand(&self, glue_crate: &Path) -> TokenStream {
            let mut withs = Vec::new();

            if let Some(tag) = &self.tag {
                withs.push(quote! { with_tag(#tag) });
            }
            if let Some(level) = &self.level {
                withs.push(quote! { with_min_level(#glue_crate::log::Level::#level) });
            }
            if let Some(filter) = &self.filter {
                withs.push(quote! {
                    with_filter(#glue_crate::android_logger::FilterBuilder::new().parse(#filter).build())
                });
            }

            quote! {
                #glue_crate::android_logger::init_once(
                    #glue_crate::android_logger::Config::default()
                    #(.#withs)*
                );
            }
        }
    }

    impl ToTokens for LogLevel {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            use LogLevel::*;

            tokens.extend(match self {
                Error => quote! { Error },
                Warn => quote! { Warn },
                Info => quote! { Info },
                Debug => quote! { Debug },
                Trace => quote! { Trace },
            });
        }
    }
}

impl MainAttr {
    pub fn expand(&self, main_fn_item: &ItemFn) -> TokenStream {
        let main_fn_name = &main_fn_item.sig.ident;
        let glue_crate = crate_path("ndk-glue", &self.ndk_glue);

        let preamble = {
            let backtrace = &self.backtrace;
            once(quote! { #backtrace })
        };

        #[cfg(feature = "logger")]
        let preamble = preamble.chain(
            self.logger
                .as_ref()
                .map(|l| l.expand(&glue_crate))
                .into_iter(),
        );

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
    fn main_with_backtrace_on() {
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
    fn main_with_backtrace_full() {
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

    #[test]
    fn main_with_overridden_ndk_glue() {
        let attr = MainAttr {
            ndk_glue: Some(parse_quote! { my::re::exported::ndk_glue }),
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
                my::re::exported::ndk_glue::init(
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
        fn main_with_logger_default() {
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
                    ndk_glue::android_logger::init_once(
                        ndk_glue::android_logger::Config::default()
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
        fn main_with_logger_with_min_level() {
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
                    ndk_glue::android_logger::init_once(
                        ndk_glue::android_logger::Config::default()
                            .with_min_level(ndk_glue::log::Level::Debug)
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
        fn main_with_logger_with_tag() {
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
                    ndk_glue::android_logger::init_once(
                        ndk_glue::android_logger::Config::default()
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
        fn main_with_logger_with_filter() {
            let attr = MainAttr {
                logger: Some(LoggerProp {
                    filter: Some("debug,hellow::world=trace".into()),
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
                    ndk_glue::android_logger::init_once(
                        ndk_glue::android_logger::Config::default()
                            .with_filter(ndk_glue::android_logger::FilterBuilder::new().parse("debug,hellow::world=trace").build())
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
        fn main_with_logger_with_min_level_and_with_tag() {
            let attr = MainAttr {
                logger: Some(LoggerProp {
                    level: Some(LogLevel::Warn),
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
                    ndk_glue::android_logger::init_once(
                        ndk_glue::android_logger::Config::default()
                            .with_tag("my-tag")
                            .with_min_level(ndk_glue::log::Level::Warn)
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
        fn main_with_backtrace_on_and_logger_with_tag() {
            let attr = MainAttr {
                backtrace: Some(BacktraceProp::On),
                logger: Some(LoggerProp {
                    tag: Some("my-tag".into()),
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
                    std::env::set_var("RUST_BACKTRACE", "1");
                    ndk_glue::android_logger::init_once(
                        ndk_glue::android_logger::Config::default()
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

        #[test]
        fn main_with_logger_with_overridden_ndk_glue_and_filter() {
            let attr = MainAttr {
                logger: Some(LoggerProp {
                    filter: Some("debug,hellow::world=trace".into()),
                    ..Default::default()
                }),
                ndk_glue: Some(parse_quote! { my::re::exported::ndk_glue }),
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
                    my::re::exported::ndk_glue::android_logger::init_once(
                        my::re::exported::ndk_glue::android_logger::Config::default()
                            .with_filter(my::re::exported::ndk_glue::android_logger::FilterBuilder::new().parse("debug,hellow::world=trace").build())
                    );
                    my::re::exported::ndk_glue::init(
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
        fn main_with_logger_with_overridden_ndk_glue_and_log_level() {
            let attr = MainAttr {
                logger: Some(LoggerProp {
                    level: Some(LogLevel::Trace),
                    ..Default::default()
                }),
                ndk_glue: Some(parse_quote! { my::re::exported::ndk_glue }),
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
                    my::re::exported::ndk_glue::android_logger::init_once(
                        my::re::exported::ndk_glue::android_logger::Config::default()
                            .with_min_level(my::re::exported::ndk_glue::log::Level::Trace)
                    );
                    my::re::exported::ndk_glue::init(
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
