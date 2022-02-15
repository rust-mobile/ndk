use core::ops::Deref;
use proc_macro2::{Ident, Span};
use proc_macro_crate::FoundCrate;
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
fn crate_name(name: &str) -> Result<FoundCrate> {
    Ok(FoundCrate::Name(name.replace('-', "_")))
}

pub fn crate_path(name: &str, overridden_path: &Option<Path>) -> Path {
    // try to use overridden crate path
    overridden_path.clone().unwrap_or_else(|| {
        Ident::new(
            // try to determine crate name from Cargo.toml
            match crate_name(name)
                .as_ref()
                .expect("Crate not found in `Cargo.toml`")
            {
                FoundCrate::Itself => "ndk_macro",
                FoundCrate::Name(n) => n.as_str(),
            },
            Span::call_site(),
        )
        .into()
    })
}
