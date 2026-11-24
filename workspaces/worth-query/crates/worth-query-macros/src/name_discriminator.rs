//! `#[derive(ApplicationNameDiscriminator)]`: a persistent-name
//! discriminator's canonical encoding from one struct or enum.
//!
//! Type keys: `identity = ".."` (the discriminator's portable identity,
//! required) and `referenced_roles = ["..", ..]` (same-occurrence roles its
//! name references are born under). Variant and named-field key: `name =
//! ".."` (the persisted name, default the Rust name). An enum writes its
//! variant's name and then its fields; a named field is written under its
//! name and a tuple field by position, exactly as a hand-written
//! `encode_canonical` over `variant`, `field` and `value` does.
//!
//! Every field must be a discriminator field: a closed value, a name
//! reference or a nested discriminator. Any other type, such as a float, is
//! refused by the compiler at the field.
use proc_macro2::TokenStream;
use syn::DeriveInput;

mod emit;
mod grammar;
#[cfg(test)]
mod tests;

pub(crate) const DERIVE: &str = "ApplicationNameDiscriminator";

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let discriminator = grammar::DiscriminatorDeclaration::parse(input)?;
    Ok(emit::discriminator(input, &discriminator))
}
