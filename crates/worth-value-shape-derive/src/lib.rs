//! Derive canonical wire encoding, portable shape, and bounded decoding together.

use proc_macro::TokenStream;

mod grammar;
mod seed;
mod serialize;
mod shape;

/// Derives `Serialize` and `CanonicalValue` from one supported wire grammar.
///
/// Supports named, unit, and newtype structs, and externally tagged enums with
/// unit or newtype variants. Only literal `serde(rename = "...")` is accepted;
/// unsupported Serde behavior is rejected rather than decoded without bounds.
#[proc_macro_derive(WorthValue, attributes(serde))]
pub fn derive_worth_value(input: TokenStream) -> TokenStream {
    let result = syn::parse::<syn::DeriveInput>(input)
        .and_then(grammar::Value::parse)
        .map(|value| {
            let serialize = serialize::emit(&value);
            let canonical = shape::emit(&value);
            quote::quote! { #serialize #canonical }
        });
    result.unwrap_or_else(syn::Error::into_compile_error).into()
}
