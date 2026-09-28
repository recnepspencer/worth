//! Public declaration-authority contract.

/// Application schema declaration: marker identity traits and typed
/// references for entities, aspects, fields, relations, operations, effects,
/// abilities, and policies, and the builder that assembles and validates an
/// `ApplicationSchemaDeclaration`.
pub mod application_schema {
    pub use crate::application_schema::*;
}

/// Aftermath declaration: what may happen to an operation's committed change
/// afterward, declared as separate correction-authority and
/// correction-mechanism axes. The published posture is derived at
/// installation.
pub mod application_aftermath {
    pub use crate::application_aftermath::*;
}

/// Capability declaration: capability, context, slot, and provenance
/// references, capability contracts, and their allow and deny, delegation,
/// elevation, revocation, and currentness rules. Capabilities are granted to principals
/// and checked at admission.
pub mod application_capability {
    pub use crate::application_capability::*;
}

/// Application query declaration: typed query definitions and their builder,
/// parameters, ordering, disclosure and live-read contracts, and the bindings
/// that tie an application request value to an installed query.
pub mod application_query {
    pub use crate::application_query::*;
}

/// Mutation binding declaration: the bindings, intents, descriptors, output
/// contracts, principal and scope contracts, and source expectations that tie
/// an application intent type to an installed operation, plus the candidate
/// ceilings a handler declares.
pub mod application_operation {
    pub use crate::application_operation::*;
}

/// Program declaration: the authored, static composition of an application
/// revision, including features, actions, outputs, managed computations,
/// workflows, and evolution between revisions.
pub mod application_program {
    pub use crate::application_program::*;
}

/// Principal identity values used by principal bindings: the external
/// principal identity carried on a mapping entity and the mapping's status.
pub mod authentication {
    pub use crate::authentication::*;
}

/// Untyped query authoring: collection and detail query and result-shape
/// builders, graph-read domain operation declarations, and the names and
/// errors they use. The `typed` module offers schema-checked builders over the
/// same meaning.
pub mod authoring {
    pub use crate::authoring::*;
}

#[doc(hidden)]
pub mod foundation {
    pub use crate::authoring::*;
}

/// Query binding slots, binding descriptors, and `resolve_bindings`, which
/// checks supplied bindings against a query's required slots: each slot bound
/// once, with the expected subject, and no undeclared slots.
pub mod binding {
    pub use crate::binding::*;
}

/// Authored meaning for creating a product branch: the fork intent that names
/// the source occurrence and the posture chosen for each component. It is
/// pure declaration and grants nothing.
pub mod branch {
    pub use crate::branch::*;
}

/// Canonicalization of an authored query bundle into its canonical query and
/// result-shape artifacts, plus the portable records used to readmit a
/// canonical bundle.
pub mod canonicalization {
    pub use crate::canonicalization::*;
}

/// Collection query planning vocabulary: ordering contracts, window and
/// cursor policy, traversal limits, and post-read shaping such as aggregates,
/// rollups, and derived fields.
pub mod collection {
    pub use crate::collection::*;
}

/// Canonicalization reports, counters, and warnings. They describe what
/// canonicalization did and grant nothing.
pub mod diagnostics {
    pub use crate::diagnostics::*;
}

pub mod domain_computation {
    pub use crate::domain_computation::*;
}

/// Digests that identify canonical and validated queries, result shapes,
/// schema bases, binding fulfillment, and collection plans. A digest is an
/// address, never permission.
pub mod identity {
    pub use crate::identity::*;
}

/// Stable semantic identities for Rust types that enter portable Query
/// meaning, and the trait that declares them.
pub mod portable_identity {
    pub use crate::portable_identity::*;
}

/// The typed carrier of canonical Query identity, minted only by
/// canonicalization. It is not installation or execution authority.
pub mod identity_authority {
    pub use crate::identity_authority::QueryCanonicalAuthority;
}

pub mod result_shape {
    pub use crate::result_shape::*;
}

/// The schema view that queries are validated against, and the typed carrier
/// of schema-basis identity minted when the view is built.
pub mod schema_view {
    pub use crate::schema_basis_authority::QuerySchemaBasisAuthority;
    pub use crate::schema_view::*;
}

/// Schema-checked query authoring: typed collection and detail query and
/// result-shape builders, and the field and relation traits that constrain
/// which predicates, orderings, and traversals compile.
pub mod typed {
    pub use crate::schema_view::{QuerySchemaView, SchemaFieldView, SchemaRelationView};
    pub use crate::typed::*;
    pub use worth_foundational::facade::ScalarAspectType;
}

/// Validation of a canonical query bundle against a schema view, producing a
/// validated bundle or a typed validation error, with its report and
/// counters.
pub mod validation {
    pub use crate::validation::*;
}

/// View-shape declaration: descriptors for how a query result is presented
/// (table, detail, inspector, grouped), and `admit_view_shape`, which checks a
/// descriptor against a canonical query bundle.
pub mod view_declaration {
    pub use crate::view_declaration::*;
}
