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
//! The relation role also accepts `cardinality = ..`, `acyclic`,
//! `self_edges = false`, `unique = ..`, `deletion = ..`, `cross_context = ..`,
//! `symmetric`, `orbit`, `same_partition_endpoints`, and
//! `reaches_at_least = ..` as native integrity declarations.
use proc_macro2::TokenStream;
use syn::DeriveInput;

pub(crate) mod emit;
pub(crate) mod grammar;
#[cfg(test)]
mod tests;

pub(crate) const DERIVE: &str = "WorthQuerySchema";

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    crate::grammar::unit_struct(input, DERIVE)?;
    let marker = grammar::SchemaMarker::parse(input)?;
    Ok(emit::marker(input, &marker))
}
