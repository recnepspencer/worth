#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! # worth-relational
//!
//! Deterministic truth-state runtime infrastructure for high-consequence graph
//! domains such as order and inventory systems, account ledgers, document
//! approval workflows, and other workloads that require durable identity,
//! transactional mutation, replay, lineage, and audit-grade diagnostics.
//!
//! The crate is intentionally shaped around the Worth domain standards:
//!
//! - component-oriented structure
//! - semantic owners for access, authority, planning, execution, and durable data
//! - a single public facade boundary
//! - contracts that preserve serialized authority and immutable read semantics
//!
//! The initial scaffold is implementation-light and contract-heavy on purpose.
//! For this runtime, getting the boundaries right early is materially more
//! important than racing toward a shallow feature-complete prototype.
//!
//! Local developer note for Milestone 2 aspect semantics:
//! `src/aspect_truth_flow.md`

#![forbid(unsafe_code)]
#![deny(unreachable_patterns)]
// These examples exercise the consumer facade. Their text also documents the Branch types.
#![cfg_attr(doctest, doc = include_str!("../../worth-relational-branch/docs/facade_examples/canonical_envelope.md"))]
#![cfg_attr(doctest, doc = include_str!("../../worth-relational-branch/docs/facade_examples/shared_canonical_envelope.md"))]
#![cfg_attr(doctest, doc = include_str!("../../worth-relational-branch/docs/facade_examples/retired_branch_basis.md"))]
#![cfg_attr(doctest, doc = include_str!("../../worth-relational-branch/docs/facade_examples/retained_fork_source.md"))]
#![cfg_attr(doctest, doc = include_str!("../../worth-relational-branch/docs/facade_examples/pinned_entity_metadata.md"))]
#![cfg_attr(doctest, doc = include_str!("../../worth-relational-branch/docs/facade_examples/caller_association.md"))]
#![cfg_attr(doctest, doc = include_str!("../../worth-relational-mutation/docs/facade_examples/authorization_durable_dependencies.md"))]
#![cfg_attr(doctest, doc = include_str!("../../worth-relational-mutation/docs/facade_examples/custom_invariant_managed_source.md"))]
#![cfg_attr(doctest, doc = include_str!("../../worth-relational-durability/docs/facade_examples/associated_effects.md"))]
#![cfg_attr(doctest, doc = include_str!("../../worth-relational-durability/docs/facade_examples/recovered_publication.md"))]
#![cfg_attr(doctest, doc = include_str!("../../worth-relational-publication/docs/facade_examples/commit_result.md"))]
#![cfg_attr(doctest, doc = include_str!("../../worth-relational-publication/docs/facade_examples/pinned_read.md"))]
#![cfg_attr(doctest, doc = include_str!("../../worth-relational-publication/docs/facade_examples/caller_association.md"))]
#![cfg_attr(doctest, doc = include_str!("../../worth-relational-publication/docs/facade_examples/predecessor_basis.md"))]

pub(crate) use worth_relational_branch::internal_port::{query, schema, snapshots, symbols};
#[cfg(test)]
pub(crate) use worth_relational_contracts::internal_port::canonical_basis_ready_sequence;
pub(crate) use worth_relational_contracts::internal_port::{
    aspect_wire, config, diagnostics, errors, graph_member, identity,
};
pub(crate) use worth_relational_mutation::internal_port::{aggregate_identity, derived_relation};
mod authority;
mod authorization;
mod branch;
mod capabilities;
mod change_source;
mod commit_strategies;
mod content;
mod durability;
mod execution;
mod history;
mod identity_authority {
    pub(crate) use worth_relational_contracts::internal_port::identity_authority::authority::relational_source_truth_authority;
    pub use worth_relational_contracts::internal_port::identity_authority::{
        RelationalBranchIdentityKind, RelationalCanonicalDigestIdentityBasis,
        RelationalCommitIdentityKind, RelationalEntityIdentityKind, RelationalRelationIdentityKind,
        RelationalSnapshotIdentityKind, RelationalSourceTruthAuthorityIdentity,
        RelationalSourceTruthBoundaryBridgedIdentity, RelationalSourceTruthDigestIdentityEvidence,
        RelationalSourceTruthExternalIdentityToken, RelationalSourceTruthProjectionIdentity,
        RelationalVersionIdentityKind, RelationalWorkspaceIdentityKind,
    };
}
mod indexes;
mod inspection;
mod lineage;
mod merge;
mod mvcc;
mod performance;
mod presentation;
mod projected_member_domain;
mod publication;
mod replay;
mod runtime;
mod simulation;
mod storage;
mod transactions;
mod validation;
mod visibility;

pub mod facade;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_execution_policy {
    pub(crate) use worth_harness::facade::execution_fixture::{
        serial_request, serial_request_with_memory,
    };
}
#[cfg(test)]
use worth_harness::facade::process_deadline;
#[cfg(test)]
#[path = "lib_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
#[cfg(test)]
mod inline_type_sizes;
