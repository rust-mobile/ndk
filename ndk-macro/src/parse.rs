use darling::FromMeta;
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

#[cfg(feature = "logger")]
mod logger {
    use super::*;

    #[derive(FromMeta, PartialEq, Eq, Default, Debug, Clone)]
    #[darling(default)]
    pub struct LoggerProp {
        // Minimum log level
        pub level: Option<LogLevel>,
        // Tag name for logger
        pub tag: Option<String>,
        // Filtering rules
        pub filter: Option<String>,
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
    fn overridden_ndk_glue() {
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

            let logger = attr.logger.unwrap();

            assert_eq!(logger.level, Some(LogLevel::Debug));
            assert_eq!(logger.tag, None);
        }

        #[test]
        fn logger_with_tag() {
            let attr: AttributeArgs = parse_quote! {
                logger(tag = "my-tag")
            };
            let attr: MainAttr = FromMeta::from_list(&attr).unwrap();

            let logger = attr.logger.unwrap();

            assert_eq!(logger.level, None);
            assert_eq!(logger.tag.unwrap(), "my-tag");
        }

        #[test]
        fn logger_with_filter() {
            let attr: AttributeArgs = parse_quote! {
                logger(filter = "debug,hello::world=trace")
            };
            let attr: MainAttr = FromMeta::from_list(&attr).unwrap();

            let logger = attr.logger.unwrap();

            assert_eq!(logger.level, None);
            assert_eq!(logger.filter.unwrap(), "debug,hello::world=trace");
        }

        #[test]
        fn logger_with_level_and_with_tag() {
            let attr: AttributeArgs = parse_quote! {
                logger(level = "error", tag = "my-app")
            };
            let attr: MainAttr = FromMeta::from_list(&attr).unwrap();

            let logger = attr.logger.unwrap();

            assert_eq!(logger.level, Some(LogLevel::Error));
            assert_eq!(logger.tag.unwrap(), "my-app");
        }

        #[test]
        fn backtrace_on_and_logger_with_level_and_with_tag() {
            let attr: AttributeArgs = parse_quote! {
                logger(level = "warn", tag = "my-app"),
                backtrace = "on"
            };
            let attr: MainAttr = FromMeta::from_list(&attr).unwrap();

            assert_eq!(attr.backtrace, Some(BacktraceProp::On));

            let logger = attr.logger.unwrap();

            assert_eq!(logger.level, Some(LogLevel::Warn));
            assert_eq!(logger.tag.unwrap(), "my-app");
        }
    }
}
