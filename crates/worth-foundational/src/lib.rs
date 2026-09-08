//! Shared WORTH boundary vocabulary.
//!
//! `worth-foundational` standardizes the meaning exchanged between WORTH
//! crates. It is not a hot-path storage runtime, mutation engine, planner, or
//! proof kernel. Domain crates may keep local optimized representations, then
//! materialize foundational boundary forms when they cross crate or artifact
//! boundaries.
//!
//! Milestone 1 begins with the crate boundary itself: all public vocabulary is
//! curated through the facade, while responsibility-shaped internal homes are
//! named before semantic value, aspect, identity, locator, compatibility, and
//! canonicalization types land.
//!
//! Crate documentation entrypoints live under `docs/README.md`.
//! Capability-specific docs currently include:
//! - `docs/aspect-contracts-values-and-authoritative-state/README.md`
//! - `docs/canonical-basis-and-reproducible-identity/README.md`
//! - `docs/profiles-and-policy-vocabulary/README.md`
//! - `docs/boundary-artifact-taxonomy-and-materialization-contracts/README.md`
//! - `docs/branching-merging-and-commit-vocabulary/README.md`
//! - `docs/diagnostics-and-explanation-ontology/README.md`
//! - `docs/lineage-provenance-receipts-and-support-truth/README.md`
//! - `docs/performance/README.md`
//! - [`physical_integrity_observation`]: portable physical facts without media,
//!   decoder, recovery, or repair authority.

#![forbid(unsafe_code)]

mod aspects;
mod boundary;
mod boundary_artifacts;
mod boundary_evidence;
pub mod boundary_evidence_api;
mod boundary_protocol;
mod canonicalization;
pub mod canonicalization_api;
mod compatibility;
mod diagnostics;
pub mod facade;
mod identities;
mod locators;
mod performance;
pub mod performance_api;
pub mod physical_integrity_observation;
mod profiles;
pub mod profiles_api;
mod responsibilities;
mod transitions;
mod values;

pub use facade::*;
pub use physical_integrity_observation::*;
