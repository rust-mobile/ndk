use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Paren,
    Result, Token,
};

use super::{MainAttr, MainProp};

pub mod keyword {
    use syn::custom_keyword as kw;

    kw!(backtrace);
    kw!(full);
}

impl MainAttr {
    pub fn backtrace_config(&self) -> Option<BacktraceConfig> {
        let mut config = None;
        for prop in &self.props {
            #[allow(irrefutable_let_patterns)]
            if let MainProp::Backtrace(MainPropBacktrace { ref props, .. }) = prop {
                if config.is_none() {
                    config = Some(BacktraceConfig::default());
                }
                config.as_mut().unwrap().merge(props);
            }
        }
        config
    }
}

pub struct MainPropBacktrace {
    #[allow(unused)]
    key: keyword::backtrace,
    #[allow(unused)]
    paren: Option<Paren>,
    props: Punctuated<BacktraceProp, Token![,]>,
}

impl Parse for MainPropBacktrace {
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

pub enum BacktraceProp {
    Full(keyword::full),
}

impl Parse for BacktraceProp {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::full) {
            Ok(BacktraceProp::Full(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BacktraceConfig {
    On,
    Full,
}

impl Default for BacktraceConfig {
    fn default() -> Self {
        Self::On
    }
}

impl<'a> From<&'a BacktraceProp> for BacktraceConfig {
    fn from(_prop: &BacktraceProp) -> Self {
        Self::Full
    }
}

impl AsRef<str> for BacktraceConfig {
    fn as_ref(&self) -> &str {
        match self {
            BacktraceConfig::On => "1",
            BacktraceConfig::Full => "full",
        }
    }
}

impl BacktraceConfig {
    pub fn merge(&mut self, props: &Punctuated<BacktraceProp, Token![,]>) {
        for prop in props {
            *self = prop.into();
        }
    }
}
