//!# A custom derive implementation for `#[derive(Logpack)]`

#![crate_type = "proc-macro"]
#![recursion_limit = "250"]

extern crate proc_macro;

mod type_derive;
mod encode_derive;
use std::process::Command;
use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as Tokens};
use syn::{DeriveInput, GenericParam, Generics};
use quote::quote;

#[proc_macro_derive(Logpack, attributes(Logpack))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let name = &input.ident;

    let a = type_derive::derive(&input);
    let b = encode_derive::derive(&input);
    let res = quote!(#a #b);

    if let Some((_, value)) =
        std::env::vars().find(|(key, _)| key.as_str() == "LOGPACK_DERIVE_SAVE_DIR")
    {
        let dir = std::path::Path::new(value.as_str());
        tokens_to_rustfmt_file(&dir.join(format!("derive_logpack_{}.rs", name)), &res);
    }

    res.into()
}

fn tokens_to_rustfmt_file(filename: &std::path::Path, expanded: &Tokens) {
    let mut file = std::fs::File::create(&filename).unwrap();
    use std::io::Write;
    file.write_all(format!("{}", expanded).as_bytes()).unwrap();
    Command::new("rustfmt")
        .args(&[filename])
        .output()
        .expect("failed to execute process");
}

fn add_trait_bounds(
    mut generics: Generics,
    skip_set: &HashSet<String>,
    trait_names: &[Tokens],
) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            if let Some(_) = skip_set.get(&type_param.ident.to_string()) {
                continue;
            }
            for trait_name in trait_names {
                let bound = syn::parse(quote! { #trait_name }.into()).unwrap();
                type_param.bounds.push(bound);
            }
            let bound = syn::parse(quote! { 'static }.into()).unwrap();
            type_param.bounds.push(bound);
        }
    }
    generics
}
