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

mod aggregate_identity;
pub(crate) use worth_relational_contracts::internal_port::aspect_wire;
mod authority;
mod authorization;
mod branch;
pub(crate) use worth_relational_contracts::internal_port::canonical_basis_ready_sequence;
mod capabilities;
mod change_source;
mod commit_strategies;
pub(crate) use worth_relational_contracts::internal_port::config;
mod content;
mod derived_relation;
pub(crate) use worth_relational_contracts::internal_port::diagnostics;
mod durability;
pub(crate) use worth_relational_contracts::internal_port::errors;
mod execution;
pub(crate) use worth_relational_contracts::internal_port::graph_member;
mod history;
pub(crate) use worth_relational_contracts::internal_port::identity;
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
pub(crate) use worth_relational_branch::internal_port::query;
mod replay;
mod runtime;
pub(crate) use worth_relational_branch::internal_port::schema;
mod simulation;
pub(crate) use worth_relational_branch::internal_port::snapshots;
mod storage;
pub(crate) use worth_relational_branch::internal_port::symbols;
pub(crate) use worth_relational_contracts::internal_port::structural_traversal;
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
