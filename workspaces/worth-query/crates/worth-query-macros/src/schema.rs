//! `#[derive(WorthQuerySchema)]`: one unit marker's schema role.
//!
//! - `schema`: `Fragment`, `ApplicationSchema` and `ApplicationSchemaComposition`
//!   for an installation root whose members are its contributions.
//! - `fragment`: `Fragment` with its binding set and members.
//! - `entity`: `Entity`, plus `reference()` with `reference(Schema, Fragment)`.
//! - `relation`: `Relation`, plus `reference()` with `from = Entity` and
//!   `reference(Schema, Fragment)`.
//!
//! `name = ".."` keeps an existing persisted name; it defaults to the type name.
//! A relation with non-default integrity constants implements `Relation` by
//! hand.
use proc_macro2::TokenStream;
use syn::DeriveInput;

mod emit;
mod grammar;
#[cfg(test)]
mod tests;

pub(crate) const DERIVE: &str = "WorthQuerySchema";

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    crate::grammar::unit_struct(input, DERIVE)?;
    let marker = grammar::SchemaMarker::parse(input)?;
    Ok(emit::marker(input, &marker))
}
