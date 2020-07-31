use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Path;

#[cfg(feature = "logger")]
pub use logger::{LogLevel, LoggerProp};

#[derive(Default, FromMeta, Debug)]
#[darling(default)]
pub struct MainAttr {
    pub backtrace: Option<BacktraceProp>,
    // Path to `ndk_glue` to override
    pub ndk_glue: Option<Path>,
    #[cfg(feature = "logger")]
    pub logger: Option<LoggerProp>,
}

#[derive(FromMeta, PartialEq, Eq, Debug, Clone, Copy)]
#[darling(default)]
pub enum BacktraceProp {
    On,
    Full,
}

impl Default for BacktraceProp {
    fn default() -> Self {
        BacktraceProp::On
    }
}

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
    use crate::helper::crate_path;
    use darling::FromMeta;
    use proc_macro2::TokenStream;
    use quote::{quote, ToTokens};
    use syn::Path;

    #[derive(FromMeta, PartialEq, Eq, Default, Debug, Clone)]
    #[darling(default)]
    pub struct LoggerProp {
        // Minimum log level
        pub level: Option<LogLevel>,
        // Tag name for logger
        pub tag: Option<String>,
        // Path to `android_logger` to override
        pub android_logger: Option<Path>,
        // Path to `log` crate to override
        pub log: Option<Path>,
    }

    impl ToTokens for LoggerProp {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let android_logger_crate = crate_path("android_logger", &self.android_logger);
            let mut withs = Vec::new();

            if let Some(tag) = &self.tag {
                withs.push(quote! { with_tag(#tag) });
            }
            if let Some(level) = &self.level {
                let log_crate = crate_path("log", &self.log);

                withs.push(quote! { with_min_level(#log_crate::Level::#level) });
            }

            tokens.extend(quote! {
                #android_logger_crate::init_once(
                    #android_logger_crate::Config::default()
                    #(.#withs)*
                );
            });
        }
    }

    #[derive(FromMeta, PartialEq, Eq, Debug, Clone, Copy)]
    #[darling(default)]
    pub enum LogLevel {
        Error,
        Warn,
        Info,
        Debug,
        Trace,
    }

    impl Default for LogLevel {
        fn default() -> Self {
            LogLevel::Error
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::AttributeArgs;
    use syn::parse_quote;

    #[test]
    fn empty_attr() {
        let attr: AttributeArgs = parse_quote! {};
        let attr: MainAttr = FromMeta::from_list(&attr).unwrap();

        assert_eq!(attr.backtrace, None);
        #[cfg(feature = "logger")]
        assert_eq!(attr.logger, None);
    }

    #[should_panic]
    #[test]
    fn invalid_attr() {
        let attr: AttributeArgs = parse_quote! {
            wrong
        };
        let _attr: MainAttr = FromMeta::from_list(&attr).unwrap();
    }

    #[test]
    fn backtrace_on() {
        let attr: AttributeArgs = parse_quote! {
            backtrace = "on"
        };
        let attr: MainAttr = FromMeta::from_list(&attr).unwrap();

        assert_eq!(attr.backtrace, Some(BacktraceProp::On));
        #[cfg(feature = "logger")]
        assert_eq!(attr.logger, None);
    }

    #[test]
    fn backtrace_full() {
        let attr: AttributeArgs = parse_quote! {
            backtrace = "full"
        };
        let attr: MainAttr = FromMeta::from_list(&attr).unwrap();

        assert_eq!(attr.backtrace, Some(BacktraceProp::Full));
        #[cfg(feature = "logger")]
        assert_eq!(attr.logger, None);
    }

    #[test]
    fn overriden_ndk_glue() {
        let attr: AttributeArgs = parse_quote! {
            ndk_glue = "my::re::exported::ndk_glue"
        };
        let attr: MainAttr = FromMeta::from_list(&attr).unwrap();

        let expected_path: Path = parse_quote! {
            my::re::exported::ndk_glue
        };

        assert_eq!(attr.ndk_glue.unwrap(), expected_path);
    }

    #[cfg(feature = "logger")]
    mod logger {
        use super::*;

        #[test]
        fn logger_with_level() {
            let attr: AttributeArgs = parse_quote! {
                logger(level = "debug")
            };
            let attr: MainAttr = FromMeta::from_list(&attr).unwrap();

            let logger = attr.logger.as_ref().unwrap();

            assert_eq!(logger.level, Some(LogLevel::Debug));
            assert_eq!(logger.tag, None);
        }

        #[test]
        fn logger_with_tag() {
            let attr: AttributeArgs = parse_quote! {
                logger(tag = "my-tag")
            };
            let attr: MainAttr = FromMeta::from_list(&attr).unwrap();

            let logger = attr.logger.as_ref().unwrap();

            assert_eq!(logger.level, None);
            assert_eq!(logger.tag.as_ref().unwrap(), "my-tag");
        }

        #[test]
        fn logger_with_level_and_with_tag() {
            let attr: AttributeArgs = parse_quote! {
                logger(level = "error", tag = "my-app")
            };
            let attr: MainAttr = FromMeta::from_list(&attr).unwrap();

            let logger = attr.logger.as_ref().unwrap();

            assert_eq!(logger.level, Some(LogLevel::Error));
            assert_eq!(logger.tag.as_ref().unwrap(), "my-app");
        }

        #[test]
        fn backtrace_on_and_logger_with_level_and_with_tag() {
            let attr: AttributeArgs = parse_quote! {
                logger(level = "warn", tag = "my-app"),
                backtrace = "on"
            };
            let attr: MainAttr = FromMeta::from_list(&attr).unwrap();

            assert_eq!(attr.backtrace, Some(BacktraceProp::On));

            let logger = attr.logger.as_ref().unwrap();

            assert_eq!(logger.level, Some(LogLevel::Warn));
            assert_eq!(logger.tag.as_ref().unwrap(), "my-app");
        }

        #[test]
        fn overriden_android_logger() {
            let attr: AttributeArgs = parse_quote! {
                logger(android_logger = "my::re::exported::android_logger")
            };
            let attr: MainAttr = FromMeta::from_list(&attr).unwrap();

            let logger = attr.logger.unwrap();

            let expected_path: Path = parse_quote! {
                my::re::exported::android_logger
            };

            assert_eq!(logger.android_logger.unwrap(), expected_path);
        }

        #[test]
        fn overriden_log_crate() {
            let attr: AttributeArgs = parse_quote! {
                logger(log = "my::re::exported::log")
            };
            let attr: MainAttr = FromMeta::from_list(&attr).unwrap();

            let logger = attr.logger.unwrap();

            let expected_path: Path = parse_quote! {
                my::re::exported::log
            };

            assert_eq!(logger.log.unwrap(), expected_path);
        }
    }
}
