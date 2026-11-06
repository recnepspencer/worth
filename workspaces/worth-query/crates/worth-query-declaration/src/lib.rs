//! Canonical Query declaration authority.
//!
//! This package owns authored intent, canonicalization, schema validation,
//! bindings, result shapes and collection declarations. Installation,
//! execution, publication and replay are downstream.
//!
//! Application membership is declared top-down through [`facade::application_schema::fragment`].
//! Aspect identities derive from qualified names; revisions derive from their
//! lowered row shapes and bindings.
//!
//! ```
//! use worth_query_declaration::facade::application_schema::{fragment::{self, *}, *};
//! struct Values;
//! impl BindingSet for Values {
//!     type Work<'a> = ValueDecodeAdmission<'a>;
//!     fn with_admission<T>(work: &mut Self::Work<'_>, run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T) -> T { run(work) }
//! }
//! impl Scalar<u64> for Values { type Binding = U64ApplicationValueBinding; }
//! struct Entry;
//! impl Entity for Entry { const NAME: &'static str = "entry"; type Aspects = (Quantity,); type Relations = (); }
//! struct Quantity;
//! impl Aspect for Quantity { const NAME: &'static str = "quantity"; type Row = u64; }
//! struct Example;
//! impl Fragment for Example { const NAME: &'static str = "example"; type Bindings = Values; type Members = (Entry,); }
//! impl ApplicationSchema for Example {
//!     const NAME: &'static str = "example";
//!     fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> { fragment::declare(Self) }
//! }
//! let declaration = Example::declaration().unwrap();
//! assert!(declaration.erased().members().iter().any(|member| matches!(member, ApplicationSchemaMember::Aspect { identity, revision, .. } if identity.0 != 0 && revision.0 != 0)));
//! ```

#![forbid(unsafe_code)]

mod portable_identity;
#[macro_use]
mod portable_identity_macro;
mod application_aftermath;
mod application_capability;
mod application_history;
mod application_operation;
mod application_program;
#[doc(hidden)]
pub mod native_input_capture;
#[macro_use]
mod workflow_macro;
#[macro_use]
mod feature_spec_macro;
mod application_query;
mod application_resource;
#[macro_use]
mod application_query_macro;
#[macro_use]
mod query_binding_macro;
#[macro_use]
mod mutation_binding_macro;
mod application_schema;
mod authentication;
mod authoring;
#[macro_use]
mod application_capability_macro;
#[macro_use]
mod principal_binding_macro;
#[macro_use]
mod application_operation_macro;
mod aspect_removal_macro;
#[macro_use]
mod value_binding_macro;
mod binary_value_binding_macro;
#[macro_use]
mod structured_value_binding_macro;
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
mod feature_spec_macro_tests;
#[cfg(test)]
mod principal_binding_tests;
#[cfg(test)]
mod typed_contract_tests;

mod basis {
    pub use crate::schema_basis_authority::QuerySchemaBasisAuthority;
}

pub mod facade;
