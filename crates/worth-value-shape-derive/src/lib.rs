//! Canonical wire derivation and independent resident-memory derivation.

use proc_macro::TokenStream;

mod grammar;
mod retained;
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

/// Derives a resident heap description, independently of wire grammar.
///
/// Every field must implement `ChargedBytes`, including Serde-skipped fields.
/// Opaque/shared owners supply their own trait implementation. The optional
/// `#[charged_bytes(crate = "path")]` selects the trait's public facade.
/// Pure value owners use `worth_foundational::facade`; the default facade is
/// `worth_execution` for execution consumers.
#[proc_macro_derive(ChargedBytes, attributes(charged_bytes))]
pub fn derive_charged_bytes(input: TokenStream) -> TokenStream {
    syn::parse::<syn::DeriveInput>(input)
        .and_then(retained::grammar::ResidentValue::parse)
        .map(|value| retained::emit::emit(&value))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
