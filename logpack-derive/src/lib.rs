//!# A custom derive implementation for `#[derive(LogpackType)]`

#![crate_type = "proc-macro"]
#![recursion_limit = "250"]

extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

mod type_derive;
mod encode_derive;
use proc_macro::TokenStream;

#[proc_macro_derive(LogpackType, attributes(LogpackType))]
pub fn derive(input: TokenStream) -> TokenStream {
    let a = type_derive::derive(&input);
    let b = encode_derive::derive(&input);
    let res = quote!(#a #b);
    res.to_string().parse().expect("Couldn't parse string to tokens")
}
