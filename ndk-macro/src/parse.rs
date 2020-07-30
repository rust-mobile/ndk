use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[cfg(feature = "logger")]
pub use logger::{LogLevel, LoggerProp};

#[derive(Default, FromMeta, Debug)]
#[darling(default)]
pub struct MainAttr {
    pub backtrace: Option<BacktraceProp>,
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
    use crate::crate_name;
    use darling::FromMeta;
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote, ToTokens};

    #[derive(FromMeta, PartialEq, Eq, Default, Debug, Clone)]
    #[darling(default)]
    pub struct LoggerProp {
        pub level: Option<LogLevel>,
        pub tag: Option<String>,
    }

    impl ToTokens for LoggerProp {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            let android_logger_crate = format_ident!(
                "{}",
                crate_name("android_logger").expect("No 'android_logger' crate found!")
            );
            let mut withs = Vec::new();

            if let Some(tag) = &self.tag {
                withs.push(quote! { with_tag(#tag) });
            }
            if let Some(level) = &self.level {
                let log_crate =
                    format_ident!("{}", crate_name("log").expect("No 'log' crate found!"));

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
        fn logger_with_level_and_tag() {
            let attr: AttributeArgs = parse_quote! {
                logger(level = "error", tag = "my-app")
            };
            let attr: MainAttr = FromMeta::from_list(&attr).unwrap();

            let logger = attr.logger.as_ref().unwrap();

            assert_eq!(logger.level, Some(LogLevel::Error));
            assert_eq!(logger.tag.as_ref().unwrap(), "my-app");
        }

        #[test]
        fn backtrace_on_and_logger_with_level_and_tag() {
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
    }
}
