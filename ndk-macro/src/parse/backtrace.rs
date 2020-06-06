use syn::{
    parse::{Parse, ParseStream},
    Result,
};

use super::MainAttr;

pub mod keyword {
    use syn::custom_keyword as kw;

    kw!(backtrace);
}

impl MainAttr {
    pub fn backtrace_enabled(&self) -> bool {
        #[cfg(not(feature = "logger"))]
        {
            !self.props.is_empty()
        }

        #[cfg(feature = "logger")]
        self.props.iter().any(|prop| {
            use super::MainProp;
            if let MainProp::Backtrace { .. } = prop {
                true
            } else {
                false
            }
        })
    }
}

pub struct MainPropBacktrace {
    #[allow(unused)]
    key: keyword::backtrace,
}

impl Parse for MainPropBacktrace {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            key: input.parse()?,
        })
    }
}
