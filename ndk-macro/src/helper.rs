use core::ops::Deref;
use syn::{
    parse::{Parse, ParseStream, Result},
    Token,
};

/// A newtype for testing
///
/// This needed because AttributeArgs from syn crate is not a newtype and does not implements `Parse` trait
#[derive(Debug)]
pub struct AttributeArgs(syn::AttributeArgs);

impl Deref for AttributeArgs {
    type Target = syn::AttributeArgs;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for AttributeArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut metas = Vec::new();

        loop {
            if input.is_empty() {
                break;
            }
            let value = input.parse()?;
            metas.push(value);
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(Self(metas))
    }
}
