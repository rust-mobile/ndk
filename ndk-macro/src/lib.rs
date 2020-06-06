use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

mod expand;
mod parse;

use parse::MainAttr;

#[cfg(not(test))]
pub(crate) use proc_macro_crate::crate_name;

#[cfg(test)]
pub(crate) fn crate_name(name: &str) -> Result<String, String> {
    Ok(name.replace('-', "_"))
}

#[proc_macro_attribute]
pub fn main(attr_input: TokenStream, item_input: TokenStream) -> TokenStream {
    let item_ast = parse_macro_input!(item_input as ItemFn);
    let attr_ast = parse_macro_input!(attr_input as MainAttr);

    attr_ast.expand(&item_ast).into()
}
