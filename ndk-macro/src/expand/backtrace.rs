use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::MainAttr;

impl MainAttr {
    pub fn expand_backtrace(&self) -> Option<TokenStream> {
        if self.backtrace_enabled() {
            Some(quote! { std::env::set_var("RUST_BACKTRACE", "1"); })
        } else {
            None
        }
    }
}
