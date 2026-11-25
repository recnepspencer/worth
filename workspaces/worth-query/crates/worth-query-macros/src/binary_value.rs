//! `#[derive(WorthBinaryValue)]`: generates a versioned, fixed-width, metered
//! `BinaryValueDelegate`, `ApplicationScalarValueBinding`,
//! `ApplicationReadableScalarValueBinding`, `WorthQueryDefaultValueBinding`,
//! and `WorthBinaryWireField` from a plain Rust struct.
mod emit;
mod grammar;
mod hygiene;
#[cfg(test)]
mod tests;

use proc_macro2::TokenStream;
use syn::DeriveInput;

pub(crate) const DERIVE: &str = "WorthBinaryValue";

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let spec = grammar::BinaryValueDeclaration::parse(input)?;
    Ok(emit::binary_value(input, &spec))
}
