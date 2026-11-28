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
//! - `WorthQueryRelation`: a standalone relation marker, its integrity
//!   constants, and its optional `reference()` constructor.
//! - `WorthQueryRecord`: an aspect marker, its `Record` shape, optional
//!   column selection aliases, and zero-string field selectors, from one row
//!   struct.
//! - `WorthQueryProjection`: the slot markers, their portable identities, the
//!   query result shape and the projection, from one result row struct.
//! - `WorthQuery`: the query marker, input binding, named parameters, scope
//!   extractor, query binding, and optional definition defaults, from one
//!   request struct.
//! - `WorthQueryAction`: the input binding, typed input-field accessors,
//!   operation, mutation binding and optional source-completion delivery, from
//!   one input struct.
//! - `WorthQueryNameRole`: a persistent-name scheme's roles, their
//!   dependencies and their canonical order, from one enum.
//! - `ApplicationNameDiscriminator`: a persistent-name discriminator's
//!   canonical encoding, from one struct or enum.
//! - `WorthQueryVocabulary`: a closed enum's durable codes and identity
//!   scalar binding.
//! - `WorthQueryAbilities`: one ability marker per variant, scoped to one
//!   entity, for role-matrix installation.
//!
//! Identities derive from an `ApplicationIdentityNamespace`; its role
//! overrides, explicit `name`, `column` and `slot` keys keep existing wire
//! identities. Malformed attributes fail with an error at the offending key.
#![forbid(unsafe_code)]

use proc_macro::TokenStream;

mod abilities;
#[cfg(test)]
mod abilities_tests;
mod action;
mod binary_value;
mod case;
mod facade_paths;
mod grammar;
mod identity_member;
mod managed_index;
mod name_discriminator;
mod name_role;
mod projection;
mod query;
mod record;
mod relation;
mod schema;
mod vocabulary;
#[cfg(test)]
mod vocabulary_tests;

/// Declares a versioned, fixed-width, metered `BinaryValueDelegate`,
/// `ApplicationScalarValueBinding`, `ApplicationReadableScalarValueBinding`,
/// `WorthQueryDefaultValueBinding`, and `WorthBinaryWireField` from a struct.
#[proc_macro_derive(WorthBinaryValue, attributes(worth_binary, worth_query))]
pub fn derive_worth_binary_value(input: TokenStream) -> TokenStream {
    expand(input, binary_value::expand)
}

/// Declares a unit struct's `ApplicationManagedIndexBinding<Schema>` with
/// checked binary-source-tree defaults and optional stock/custom provider wiring.
#[proc_macro_derive(WorthManagedIndex, attributes(worth_index, worth_query))]
pub fn derive_worth_managed_index(input: TokenStream) -> TokenStream {
    expand(input, managed_index::expand)
}

/// Declares a unit struct's schema role: `schema` (an installation root),
/// `fragment`, `entity` or `relation`.
#[proc_macro_derive(WorthQuerySchema, attributes(worth_query))]
pub fn derive_worth_query_schema(input: TokenStream) -> TokenStream {
    expand(input, schema::expand)
}

/// Declares a unit struct's `Relation` trait, integrity constants and optional
/// `reference()` constructor.
#[proc_macro_derive(WorthQueryRelation, attributes(worth_query))]
pub fn derive_worth_query_relation(input: TokenStream) -> TokenStream {
    expand(input, relation::expand)
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

/// Alias for [`WorthQueryProjection`]: declares a typed query result row's
/// slots, result shape, projection, and self-binding for `result = Row`.
#[proc_macro_derive(WorthQueryRow, attributes(worth_query))]
pub fn derive_worth_query_row(input: TokenStream) -> TokenStream {
    expand(input, projection::expand)
}

/// Declares a typed application query, its parameter references, its request
/// binding and optional definition defaults from one request struct.
#[proc_macro_derive(WorthQuery, attributes(worth_query))]
pub fn derive_worth_query(input: TokenStream) -> TokenStream {
    expand(input, query::expand)
}

/// Declares a mutation action from its input struct.
#[proc_macro_derive(WorthQueryAction, attributes(worth_query))]
pub fn derive_worth_query_action(input: TokenStream) -> TokenStream {
    expand(input, action::expand)
}

/// Declares a persistent-name scheme's roles from an enum and orders it by
/// canonical role order, then numeric index.
#[proc_macro_derive(WorthQueryNameRole, attributes(worth_query, depends_on))]
pub fn derive_worth_query_name_role(input: TokenStream) -> TokenStream {
    expand(input, name_role::expand)
}

/// Declares a persistent-name discriminator's canonical encoding from its
/// fields or variants.
#[proc_macro_derive(ApplicationNameDiscriminator, attributes(worth_query))]
pub fn derive_application_name_discriminator(input: TokenStream) -> TokenStream {
    expand(input, name_discriminator::expand)
}

/// Declares a closed, code-identified enum's identity scalar binding and its
/// `ApplicationValueVocabulary`, from explicit variant codes.
#[proc_macro_derive(WorthQueryVocabulary, attributes(worth_query))]
pub fn derive_worth_query_vocabulary(input: TokenStream) -> TokenStream {
    expand(input, vocabulary::expand)
}

/// Declares one ability marker per unit variant, all scoped to one entity, and
/// the enum's `ApplicationAbilityVocabulary`.
#[proc_macro_derive(WorthQueryAbilities, attributes(worth_query))]
pub fn derive_worth_query_abilities(input: TokenStream) -> TokenStream {
    expand(input, abilities::expand)
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
