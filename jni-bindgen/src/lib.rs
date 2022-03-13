use anyhow::Result;
use cafebabe::{ClassAccessFlags, MethodAccessFlags};
use heck::{ToSnakeCase, ToUpperCamelCase};
use jni::signature::{JavaType, Primitive, TypeSignature};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use zip::ZipArchive;

pub fn jni_bindgen(path: &Path, ignore: BTreeSet<&'static str>) -> Result<TokenStream> {
    let mut jar = ZipArchive::new(BufReader::new(File::open(path)?))?;
    let mut buffer = Vec::with_capacity(4096);
    let mut module = Module::default();
    for i in 0..jar.len() {
        let mut entry = jar.by_index(i)?;
        let name = entry.name();
        if !name.ends_with(".class") {
            continue;
        }
        if ignore.contains(name.strip_suffix(".class").unwrap()) {
            continue;
        }
        buffer.clear();
        entry.read_to_end(&mut buffer)?;
        let class = cafebabe::parse_class(&buffer).map_err(|err| anyhow::anyhow!("{}", err))?;
        if !class.access_flags.contains(ClassAccessFlags::PUBLIC) {
            continue;
        }
        let struct_ = module.lookup(&class.this_class);
        for method in &class.methods {
            if !method.access_flags.contains(MethodAccessFlags::PUBLIC) {
                continue;
            }
            let is_static = method.access_flags.contains(MethodAccessFlags::STATIC);
            struct_.method(&method.name, &method.descriptor, is_static)?;
        }
    }
    Ok(module.emit())
}

#[derive(Default)]
pub struct Module {
    structs: BTreeMap<String, Struct>,
    modules: BTreeMap<String, Module>,
}

impl Module {
    pub fn lookup(&mut self, obj: &str) -> &mut Struct {
        let mut submodule = self;
        let class_name = if let Some((modules, class_name)) = obj.rsplit_once('/') {
            for module in modules.split('/') {
                submodule = submodule.module(module);
            }
            class_name
        } else {
            obj
        };
        submodule.struct_(&class_name)
    }

    pub fn module(&mut self, module: &str) -> &mut Module {
        self.modules.entry(module_name(module)).or_default()
    }

    pub fn struct_(&mut self, name: &str) -> &mut Struct {
        self.structs.entry(struct_name(name)).or_default()
    }

    pub fn emit(&self) -> TokenStream {
        self.structs
            .iter()
            .map(|(name, s)| {
                let s = s.emit(&name);
                let ident = format_ident!("{}", name);
                quote! {
                    pub struct #ident<'a> {
                        env: jni::JNIEnv<'a>,
                        obj: jni::objects::JObject<'a>,
                    }

                    impl<'a> From<#ident<'a>> for jni::objects::JObject<'a> {
                        fn from(obj: #ident<'a>) -> Self {
                            obj.obj
                        }
                    }

                    impl<'a> From<(jni::JNIEnv<'a>, jni::objects::JObject<'a>)> for #ident<'a> {
                        fn from(vals: (jni::JNIEnv<'a>, jni::objects::JObject<'a>)) -> Self {
                            Self { env: vals.0, obj: vals.1 }
                        }
                    }

                    #s
                }
            })
            .chain(self.modules.iter().map(|(name, m)| {
                let m = m.emit();
                let ident = format_ident!("{}", name);
                quote! {
                    pub mod #ident {
                        #m
                    }
                }
            }))
            .collect()
    }
}

#[derive(Default)]
pub struct Struct {
    methods: BTreeMap<String, Method>,
}

impl Struct {
    pub fn method(&mut self, name: &str, descriptor: &str, is_static: bool) -> Result<()> {
        let name = method_name(name);
        let sig = TypeSignature::from_str(descriptor)?;
        let method = Method {
            sig,
            java_name: name.to_string(),
            java_desc: descriptor.to_string(),
            is_static,
        };
        self.methods.insert(name, method);
        Ok(())
    }

    pub fn emit(&self, name: &str) -> TokenStream {
        let struct_ident = format_ident!("{}", name);
        self.methods
            .iter()
            .map(|(name, method)| {
                let method_ident = format_ident!("{}", name);
                let java_name = &method.java_name;
                let java_desc = &method.java_desc;
                let ret_ty = java_type(&method.sig.ret);
                let ret_f = match method.sig.ret {
                    JavaType::Primitive(Primitive::Boolean) => quote!(z),
                    JavaType::Primitive(Primitive::Byte) => quote!(b),
                    JavaType::Primitive(Primitive::Char) => quote!(c),
                    JavaType::Primitive(Primitive::Double) => quote!(d),
                    JavaType::Primitive(Primitive::Float) => quote!(f),
                    JavaType::Primitive(Primitive::Int) => quote!(i),
                    JavaType::Primitive(Primitive::Long) => quote!(j),
                    JavaType::Primitive(Primitive::Short) => quote!(s),
                    JavaType::Primitive(Primitive::Void) => quote!(v),
                    _ => quote!(l),
                };
                let ret = if let JavaType::Object(_) = method.sig.ret {
                    quote!(Ok((self.env, res).into()))
                } else {
                    quote!(Ok(res))
                };
                let arg_decl = method.sig.args.iter().enumerate().map(|(i, ty)| {
                    let ident = format_ident!("arg{}", i);
                    let ty = java_type(ty);
                    quote!(#ident: #ty,)
                });
                let arg_ident = method.sig.args.iter().enumerate().map(|(i, _)| {
                    let ident = format_ident!("arg{}", i);
                    quote!(#ident.into(),)
                });
                if method.is_static {
                    quote! {
                        impl<'a> #struct_ident<'a> {
                            pub fn #method_ident(&self, #(#arg_decl)*) -> jni::errors::Result<#ret_ty> {
                                let res = self.env.call_method(
                                    self.obj,
                                    #java_name,
                                    #java_desc,
                                    &[#(#arg_ident)*],
                                )?
                                .#ret_f()?;
                                #ret
                            }
                        }
                    }
                } else {
                    // TODO:
                    quote!()
                }
            })
            .collect()
    }
}

pub struct Method {
    pub sig: TypeSignature,
    pub java_name: String,
    pub java_desc: String,
    pub is_static: bool,
}

fn java_type(ty: &JavaType) -> TokenStream {
    match ty {
        JavaType::Primitive(Primitive::Boolean) => quote!(bool),
        JavaType::Primitive(Primitive::Byte) => quote!(jni::sys::jbyte),
        JavaType::Primitive(Primitive::Char) => quote!(jni::sys::jchar),
        JavaType::Primitive(Primitive::Double) => quote!(jni::sys::jdouble),
        JavaType::Primitive(Primitive::Float) => quote!(jni::sys::jfloat),
        JavaType::Primitive(Primitive::Int) => quote!(jni::sys::jint),
        JavaType::Primitive(Primitive::Long) => quote!(jni::sys::jlong),
        JavaType::Primitive(Primitive::Short) => quote!(jni::sys::jshort),
        JavaType::Primitive(Primitive::Void) => quote!(()),
        JavaType::Object(name) => {
            let path = rust_path(name)
                .into_iter()
                .map(|ident| format_ident!("{}", ident));
            quote!(crate::#(#path)::*)
        }
        _ => quote!(jni::objects::JObject),
    }
}

fn rust_path(obj: &str) -> Vec<String> {
    let mut path = vec![];
    let class_name = if let Some((modules, class_name)) = obj.rsplit_once('/') {
        for module in modules.split('/') {
            path.push(module_name(module));
        }
        class_name
    } else {
        obj
    };
    path.push(struct_name(class_name));
    path
}

fn module_name(name: &str) -> String {
    sanitize_identifier(&name.to_snake_case())
}

fn struct_name(name: &str) -> String {
    sanitize_identifier(&name.replace('$', "").to_upper_camel_case())
}

fn method_name(name: &str) -> String {
    sanitize_identifier(&name.to_snake_case())
}

fn sanitize_identifier(id: &str) -> String {
    if RESERVED_IDENTIFIERS.contains(&id) {
        format!("{}_", id)
    } else {
        id.to_string()
    }
}

const RESERVED_IDENTIFIERS: &[&'static str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "Self", "self", "static", "struct", "super", "trait", "true", "type", "union",
    "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
];

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use ndk_build::ndk::Ndk;

    #[test]
    fn test_bindgen() -> Result<()> {
        let ndk = Ndk::from_env()?;
        let target_platform = ndk.default_target_platform();
        let jar = ndk.android_jar(target_platform)?;
        jni_bindgen(&jar, Default::default())?;
        Ok(())
    }
}
