//! Canonical Query declaration authority.
//!
//! This package owns authored intent, canonicalization, schema-visible
//! validation, binding grammar, result shapes, and collection declarations.
//! Runtime installation, execution, publication, and replay are downstream.
//!
//! Application aspects are declaration-owned identities. The stable
//! [`worth_query_aspect!`] form requires both an explicit
//! [`facade::application_schema::AspectIdentity`] and
//! [`facade::application_schema::AspectContractRevision`]; installation retains
//! that exact contract rather than deriving identity from a display name.
//!
//! ```
//! use worth_query_declaration::{worth_query_aspect, facade::application_schema::{
//!     AspectContractRevision, AspectIdentity,
//! }};
//!
//! # pub struct ExampleSchema;
//! # pub struct Account;
//! worth_query_aspect!(
//!     pub AccountFacts in ExampleSchema, Account;
//!     identity = AspectIdentity(0x9161_1001),
//!     revision = AspectContractRevision(1),
//! );
//!
//! assert_eq!(AccountFacts::reference().identity(), AspectIdentity(0x9161_1001));
//! assert_eq!(
//!     AccountFacts::reference().revision(),
//!     AspectContractRevision(1),
//! );
//! ```

#![forbid(unsafe_code)]

mod portable_identity;
#[macro_use]
mod portable_identity_macro;
mod application_aftermath;
mod application_capability;
mod application_query;
#[macro_use]
mod application_query_macro;
mod application_schema;
mod authentication;
mod authoring;
#[macro_use]
mod application_aspect_macro;
#[macro_use]
mod application_capability_macro;
#[macro_use]
mod application_schema_macro;
#[macro_use]
mod application_operation_macro;
mod binding;
mod branch;
mod canonicalization;
mod collection;
mod diagnostics;
mod domain_computation;
mod identity;
#[path = "canonical_authority.rs"]
mod identity_authority;
#[doc(hidden)]
pub mod lifecycle_effect_derivation_authority;
mod result_shape;
#[macro_use]
mod schema_macro;
#[path = "schema_basis_authority.rs"]
mod schema_basis_authority;
mod schema_view;
mod typed;
mod validation;
mod view_declaration;

#[cfg(test)]
mod ability_declaration_tests;
#[cfg(test)]
mod application_schema_tests;
#[cfg(test)]
mod authoring_contract_tests;
#[cfg(test)]
mod binding_contract_tests;
#[cfg(test)]
mod canonical_identity_tests;
#[cfg(test)]
mod canonicalization_normalization_tests;
#[cfg(test)]
mod principal_binding_tests;
#[cfg(test)]
mod typed_contract_tests;

mod basis {
    pub use crate::schema_basis_authority::QuerySchemaBasisAuthority;
}

pub mod facade;
