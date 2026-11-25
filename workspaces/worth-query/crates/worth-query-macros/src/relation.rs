//! `#[derive(WorthQueryRelation)]`: one unit marker's `Relation` trait,
//! integrity constants, and optional `reference()` constructor.
//!
//! Keys: `to = Entity` (required), optional `name = ".."` (default the type
//! name), optional `from = Entity` with `reference(Schema, Fragment)`, and the
//! native relation integrity keys (`cardinality`, `acyclic`, `self_edges`,
//! `unique`, `deletion`, `cross_context`, `symmetric`, `orbit`,
//! `same_partition_endpoints`, `reaches_at_least`).
use proc_macro2::TokenStream;
use syn::DeriveInput;

mod grammar;
#[cfg(test)]
mod tests;

pub(crate) const DERIVE: &str = "WorthQueryRelation";

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    crate::grammar::unit_struct(input, DERIVE)?;
    let declaration = grammar::RelationDeclaration::parse(input)?;
    let name = declaration
        .name
        .unwrap_or_else(|| syn::LitStr::new(&input.ident.to_string(), input.ident.span()));
    Ok(crate::schema::emit::relation(
        input,
        &name,
        &declaration.to,
        declaration.reference.as_ref(),
        &declaration.laws,
    ))
}
