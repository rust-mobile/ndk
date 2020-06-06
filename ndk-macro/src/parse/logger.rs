use syn::{
    parenthesized,
    parse::{Lookahead1, Parse, ParseStream},
    punctuated::Punctuated,
    token::Paren,
    LitStr, Result, Token,
};

use super::{MainAttr, MainProp};

pub mod keyword {
    use syn::custom_keyword as kw;

    kw!(logger);
    kw!(error);
    kw!(warn);
    kw!(info);
    kw!(debug);
    kw!(trace);
}

impl MainAttr {
    pub fn logger_config(&self) -> Option<LoggerConfig> {
        let mut config = None;
        for prop in &self.props {
            if let MainProp::Logger(MainPropLogger { ref props, .. }) = prop {
                if config.is_none() {
                    config = Some(LoggerConfig::default());
                }
                config.as_mut().unwrap().merge(props);
            }
        }
        config
    }
}

pub struct MainPropLogger {
    #[allow(unused)]
    key: keyword::logger,
    #[allow(unused)]
    paren: Option<Paren>,
    props: Punctuated<LoggerProp, Token![,]>,
}

impl Parse for MainPropLogger {
    fn parse(input: ParseStream) -> Result<Self> {
        let key = input.parse()?;
        let (paren, props) = if input.peek(Paren) {
            let content;
            (
                Some(parenthesized!(content in input)),
                Punctuated::parse_terminated(&content)?,
            )
        } else {
            (None, Punctuated::default())
        };

        Ok(Self { key, paren, props })
    }
}

pub enum LoggerProp {
    Level(LogLevel),
    Tag(LitStr),
}

impl Parse for LoggerProp {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if LogLevel::peek(&lookahead) {
            Ok(LoggerProp::Level(input.parse()?))
        } else if lookahead.peek(LitStr) {
            Ok(LoggerProp::Tag(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

pub enum LogLevel {
    Error(keyword::error),
    Warn(keyword::warn),
    Info(keyword::info),
    Debug(keyword::debug),
    Trace(keyword::trace),
}

impl<'a> Into<log::Level> for &'a LogLevel {
    fn into(self) -> log::Level {
        use log::Level::*;
        match self {
            LogLevel::Error(_) => Error,
            LogLevel::Warn(_) => Warn,
            LogLevel::Info(_) => Info,
            LogLevel::Debug(_) => Debug,
            LogLevel::Trace(_) => Trace,
        }
    }
}

impl LogLevel {
    pub fn peek(lookahead: &Lookahead1) -> bool {
        lookahead.peek(keyword::error)
            || lookahead.peek(keyword::warn)
            || lookahead.peek(keyword::info)
            || lookahead.peek(keyword::debug)
            || lookahead.peek(keyword::trace)
    }
}

impl Parse for LogLevel {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::error) {
            Ok(LogLevel::Error(input.parse()?))
        } else if lookahead.peek(keyword::warn) {
            Ok(LogLevel::Warn(input.parse()?))
        } else if lookahead.peek(keyword::info) {
            Ok(LogLevel::Info(input.parse()?))
        } else if lookahead.peek(keyword::debug) {
            Ok(LogLevel::Debug(input.parse()?))
        } else if lookahead.peek(keyword::trace) {
            Ok(LogLevel::Trace(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Default)]
pub struct LoggerConfig {
    pub level: Option<log::Level>,
    pub tag: Option<String>,
}

impl LoggerConfig {
    pub fn merge(&mut self, props: &Punctuated<LoggerProp, Token![,]>) {
        for prop in props {
            match prop {
                LoggerProp::Level(level) => self.level = Some(level.into()),
                LoggerProp::Tag(tag) => self.tag = Some(tag.value()),
            }
        }
    }
}
