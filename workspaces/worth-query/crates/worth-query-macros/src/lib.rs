//! Derive syntax over Query's public declaration and host projection traits.
//!
//! Applications use these derives through `worth_query_decl::facade` and
//! `worth_query_host::facade`, never by depending on this crate. Every derive
//! expands into the same public trait impls and declaration macros a
//! hand-written declaration uses, so derived and hand-written members mix
//! freely in one schema. Expansions name `::worth_query_decl` and, for
//! projections, `::worth_query_host`; an application depends on both.
//!
//! - `WorthQuerySchema`: one marker's schema role (`schema`, `fragment`,
//!   `entity` or `relation`).
//! - `WorthQueryRecord`: an aspect marker, its `Record` shape and optional
//!   column selection aliases, from one row struct.
//! - `WorthQueryProjection`: the slot markers, their portable identities, the
//!   query result shape and the projection, from one result row struct.
//! - `WorthQueryAction`: the input binding, operation, mutation binding and
//!   optional source-completion delivery, from one input struct.
//!
//! Identities derive from an `ApplicationIdentityNamespace`; its role
//! overrides, explicit `name`, `column` and `slot` keys keep existing wire
//! identities. Malformed attributes fail with an error at the offending key.
#![forbid(unsafe_code)]

use proc_macro::TokenStream;

mod action;
mod case;
mod facade_paths;
mod grammar;
mod projection;
mod record;
mod schema;

/// Declares a unit struct's schema role: `schema` (an installation root),
/// `fragment`, `entity` or `relation`.
#[proc_macro_derive(WorthQuerySchema, attributes(worth_query))]
pub fn derive_worth_query_schema(input: TokenStream) -> TokenStream {
    expand(input, schema::expand)
}

/// Declares an aspect, its record shape and its column selections from one
/// row struct.
#[proc_macro_derive(WorthQueryRecord, attributes(worth_query))]
pub fn derive_worth_query_record(input: TokenStream) -> TokenStream {
    expand(input, record::expand)
}

/// Declares a query result row's slots, result shape and projection.
#[proc_macro_derive(WorthQueryProjection, attributes(worth_query))]
pub fn derive_worth_query_projection(input: TokenStream) -> TokenStream {
    expand(input, projection::expand)
}

/// Declares a mutation action from its input struct.
#[proc_macro_derive(WorthQueryAction, attributes(worth_query))]
pub fn derive_worth_query_action(input: TokenStream) -> TokenStream {
    expand(input, action::expand)
}

fn expand(
    input: TokenStream,
    derive: fn(&syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream>,
) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    derive(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
