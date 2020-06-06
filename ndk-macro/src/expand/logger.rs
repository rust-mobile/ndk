use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{crate_name, parse::MainAttr};

impl MainAttr {
    pub fn expand_logger(&self) -> Option<TokenStream> {
        self.logger_config().map(|config| {
            let android_logger_crate = format_ident!(
                "{}",
                crate_name("android_logger").expect("No 'android_logger' crate found!")
            );
            let mut with_key = Vec::new();
            let mut with_value = Vec::new();

            if let Some(tag) = &config.tag {
                with_key.push(quote! { with_tag });
                with_value.push(quote! { #tag });
            }
            if let Some(level) = &config.level {
                let log_crate =
                    format_ident!("{}", crate_name("log").expect("No 'log' crate found!"));
                let level = level_to_token(level);

                with_key.push(quote! { with_min_level });
                with_value.push(quote! { #log_crate::Level::#level });
            }

            quote! {
                #android_logger_crate::init_once(
                    #android_logger_crate::Config::default()
                    #(. #with_key ( #with_value ) )*
                );
            }
        })
    }
}

fn level_to_token(level: &log::Level) -> TokenStream {
    match level {
        log::Level::Error => quote! { Error },
        log::Level::Warn => quote! { Warn },
        log::Level::Info => quote! { Info },
        log::Level::Debug => quote! { Debug },
        log::Level::Trace => quote! { Trace },
    }
}
