//! Canonical wire derivation and independent resident-memory derivation.

use proc_macro::TokenStream;

mod external_payload;
mod grammar;
mod retained;
mod seed;
mod serialize;
mod shape;

/// Derives `Serialize` and `CanonicalValue` from one supported wire grammar.
///
/// Supports concrete named/unit/newtype values, guarded recursion, Vec/Option/
/// Box, fixed arrays/pairs and checked platform indices, external unit/newtype/
/// named variants and internally tagged unit/named variants. Named fields can
/// select an explicit `#[worth_value(delegate = OwnerDelegate)]` admitted seed.
/// Literal rename, snake_case enum renaming and strict
/// unknown fields are supported; other Serde behavior fails at the declaration.
#[proc_macro_derive(WorthValue, attributes(serde, worth_value))]
pub fn derive_worth_value(input: TokenStream) -> TokenStream {
    let result = syn::parse::<syn::DeriveInput>(input)
        .and_then(grammar::Value::parse)
        .map(|value| {
            let payloads = external_payload::emit(&value);
            let serialize = serialize::emit(&value);
            let canonical = shape::emit(&value);
            quote::quote! { #payloads #serialize #canonical }
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
