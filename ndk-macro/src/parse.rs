mod backtrace;

#[cfg(feature = "logger")]
mod logger;

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Result, Token,
};

use backtrace::*;

#[cfg(feature = "logger")]
use logger::*;

pub struct MainAttr {
    props: Punctuated<MainProp, Token![,]>,
}

impl Parse for MainAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            props: Punctuated::parse_terminated(input)?,
        })
    }
}

pub enum MainProp {
    Backtrace(MainPropBacktrace),

    #[cfg(feature = "logger")]
    Logger(MainPropLogger),
}

impl Parse for MainProp {
    fn parse(input: ParseStream) -> Result<Self> {
        #[cfg(feature = "logger")]
        {
            let lookahead = input.lookahead1();
            if lookahead.peek(backtrace::keyword::backtrace) {
                input.parse().map(MainProp::Backtrace)
            } else if lookahead.peek(logger::keyword::logger) {
                input.parse().map(MainProp::Logger)
            } else {
                Err(lookahead.error())
            }
        }

        #[cfg(not(feature = "logger"))]
        {
            let lookahead = input.lookahead1();
            if lookahead.peek(backtrace::keyword::backtrace) {
                input.parse().map(MainProp::Backtrace)
            } else {
                Err(lookahead.error())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn empty_attr() {
        let attr: MainAttr = parse_quote! {};

        assert_eq!(attr.props.len(), 0);
        assert_eq!(attr.backtrace_config(), None);
    }

    #[should_panic]
    #[test]
    fn invalid_attr() {
        let _attr: MainAttr = parse_quote! { wrong };
    }

    #[test]
    fn single_backtrace_prop_without_props() {
        let attr: MainAttr = parse_quote! { backtrace };

        assert_eq!(attr.props.len(), 1);
        assert_eq!(attr.backtrace_config(), Some(BacktraceConfig::On));
    }

    #[test]
    fn single_backtrace_prop_with_full_prop() {
        let attr: MainAttr = parse_quote! { backtrace(full) };

        assert_eq!(attr.props.len(), 1);
        assert_eq!(attr.backtrace_config(), Some(BacktraceConfig::Full));
    }

    #[test]
    fn repeated_backtrace_props_without_props() {
        let attr: MainAttr = parse_quote! { backtrace, backtrace };

        assert_eq!(attr.props.len(), 2);
        assert_eq!(attr.backtrace_config(), Some(BacktraceConfig::On));
    }

    #[test]
    fn repeated_backtrace_props_with_props() {
        let attr: MainAttr = parse_quote! { backtrace, backtrace(full) };

        assert_eq!(attr.props.len(), 2);
        assert_eq!(attr.backtrace_config(), Some(BacktraceConfig::Full));
    }

    #[cfg(feature = "logger")]
    mod logger {
        use super::*;

        #[test]
        fn single_log_prop_with_level_prop() {
            let attr: MainAttr = parse_quote! { logger(debug) };

            assert_eq!(attr.props.len(), 1);

            let config = attr.logger_config().unwrap();

            assert_eq!(config.level, Some(log::Level::Debug));
            assert_eq!(config.tag, None);
        }

        #[test]
        fn single_log_prop_with_tag_prop() {
            let attr: MainAttr = parse_quote! { logger("my-tag") };

            assert_eq!(attr.props.len(), 1);

            let config = attr.logger_config().unwrap();

            assert_eq!(config.level, None);
            assert_eq!(config.tag.unwrap(), "my-tag");
        }

        #[test]
        fn single_log_prop_with_level_and_tag_prop() {
            let attr: MainAttr = parse_quote! { logger(error, "my-app") };

            assert_eq!(attr.props.len(), 1);

            let config = attr.logger_config().unwrap();

            assert_eq!(config.level, Some(log::Level::Error));
            assert_eq!(config.tag.unwrap(), "my-app");
        }

        #[test]
        fn single_log_prop_with_level_and_tag_prop_and_backtrace_prop() {
            let attr: MainAttr = parse_quote! { logger(error, "my-app"), backtrace };

            assert_eq!(attr.props.len(), 2);
            assert_eq!(attr.backtrace_config(), Some(BacktraceConfig::On));

            let config = attr.logger_config().unwrap();

            assert_eq!(config.level, Some(log::Level::Error));
            assert_eq!(config.tag.unwrap(), "my-app");
        }

        #[test]
        fn multiple_log_props_with_level_and_tag_prop() {
            let attr: MainAttr = parse_quote! { logger(error), logger("my-app") };

            assert_eq!(attr.props.len(), 2);

            let config = attr.logger_config().unwrap();

            assert_eq!(config.level, Some(log::Level::Error));
            assert_eq!(config.tag.unwrap(), "my-app");
        }

        #[test]
        fn multiple_log_props_with_level_and_tag_prop_with_override() {
            let attr: MainAttr =
                parse_quote! { logger(error), logger("my-app"), logger(info, "some-other") };

            assert_eq!(attr.props.len(), 3);

            let config = attr.logger_config().unwrap();

            assert_eq!(config.level, Some(log::Level::Info));
            assert_eq!(config.tag.unwrap(), "some-other");
        }
    }
}
