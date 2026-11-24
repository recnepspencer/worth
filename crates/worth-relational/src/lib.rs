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
mod query;
mod replay;
mod runtime;
mod schema;
mod simulation;
mod snapshots;
mod storage;
pub(crate) use worth_relational_contracts::internal_port::structural_traversal;
mod symbols;
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
