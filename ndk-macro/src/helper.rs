use core::ops::Deref;
use proc_macro2::{Ident, Span};
use syn::{
    parse::{Parse, ParseStream, Result},
    Path, Token,
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

#[cfg(not(test))]
use proc_macro_crate::crate_name;

#[cfg(test)]
fn crate_name(name: &str) -> Result<String> {
    Ok(name.replace('-', "_"))
}

pub fn crate_path(name: &str, overriden_path: &Option<Path>) -> Path {
    // try to use overriden crate path
    overriden_path.clone().unwrap_or_else(|| {
        // the binding to hold string from `crate_name` fn
        let mut detected_name = None;
        Ident::new(
            // try to determine crate name from Cargo.toml
            crate_name(name)
                .ok()
                .map(|name| {
                    detected_name = Some(name);
                    detected_name.as_ref().unwrap().as_str()
                })
                // or use default crate name
                // (this may cause compilation error when crate is not found)
                .unwrap_or_else(|| name),
            Span::call_site(),
        )
        .into()
    })
}
