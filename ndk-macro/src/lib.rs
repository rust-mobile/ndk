use darling::FromMeta;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

mod expand;
mod helper;
mod parse;

use helper::AttributeArgs;
use parse::MainAttr;

#[proc_macro_attribute]
pub fn main(attr_input: TokenStream, item_input: TokenStream) -> TokenStream {
    let item_ast = parse_macro_input!(item_input as ItemFn);
    let attr_ast = parse_macro_input!(attr_input as AttributeArgs);
    let attr: MainAttr = match FromMeta::from_list(&attr_ast) {
        Ok(attr) => attr,
        Err(errs) => return TokenStream::from(errs.write_errors()),
    };

    attr.expand(&item_ast).into()
}
