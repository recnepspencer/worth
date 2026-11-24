//! `#[derive(WorthQueryProjection)]`: a query result row's slots, result
//! shape and projection from one struct.
//!
//! Struct keys: `schema = Schema`, `root = Entity` (the result's root entity)
//! and `namespace = NAMESPACE`, from which each slot's identity derives as
//! `NAMESPACE.slot.<name>`. Field keys: `field = Selection` (a
//! `FieldSelection` type whose name is its column), `column = ".."` when the
//! column is named otherwise, `name = ".."` (the output name, default the
//! field name) and `slot = ".."`, an explicit identity that keeps an existing
//! wire identity. An `Option<T>` field projects an optional slot.
//!
//! A field marked `#[worth_query(name)]` projects the persistent name its
//! row's root entity carries, as `WorthQueryName<Root>`; an
//! `Option<WorthQueryName<Root>>` field projects `None` for an entity that
//! carries no name. It is the row's own identity, not a declared result slot,
//! so it takes no other key and adds nothing to the result shape.
//!
//! The derive declares a `<row>_slots` module of slot markers, their
//! `WorthQueryPortableType` impls, one query-generic selector per field, a
//! query-generic `result_shape()`, `PROJECTS_PERSISTENT_NAMES` (whether this
//! row or a related row projects a name) and `WorthQueryApplicationProjection`
//! for every query whose result binding carries this row.
use proc_macro2::TokenStream;
use syn::DeriveInput;

mod emit;
mod grammar;
#[cfg(test)]
mod tests;

pub(crate) const DERIVE: &str = "WorthQueryProjection";

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let fields = crate::grammar::named_fields(input, DERIVE)?;
    let projection = grammar::ProjectionDeclaration::parse(input, fields)?;
    Ok(emit::projection(input, &projection))
}
