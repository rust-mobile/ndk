use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::MainAttr;

impl MainAttr {
    pub fn expand_backtrace(&self) -> Option<TokenStream> {
        self.backtrace_config().map(|config| {
            let config: String = config.as_ref().into();
            quote! { std::env::set_var("RUST_BACKTRACE", #config); }
        })
    }
}
