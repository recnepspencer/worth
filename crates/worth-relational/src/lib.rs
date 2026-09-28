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

mod aspect_wire;
mod authority;
mod authorization;
mod branch;
mod canonical_basis_ready_sequence;
mod capabilities;
mod commit_strategies;
mod config;
mod diagnostics;
mod durability;
mod errors;
mod history;
mod identity;
mod identity_authority;
mod indexes;
mod inspection;
mod lineage;
mod merge;
mod mvcc;
mod performance;
mod presentation;
mod publication;
mod query;
mod replay;
mod runtime;
mod schema;
mod simulation;
mod snapshots;
mod storage;
mod symbols;
mod transactions;
mod validation;
mod visibility;

pub mod facade;

#[cfg(test)]
mod tests;
