//! Branch structural-sharing observation: the module family root.
//!
//! This file names the family's parts and re-exports them. It declares no
//! type, no constant, and no behaviour of its own, so that each named
//! responsibility keeps exactly one home and exactly one reachable path.
//!
//! Vocabulary:
//!
//! - `sharing_inspection_denial.rs` — why an inspection was refused.
//! - `sharing_byte_metric_scope.rs` — the versioned byte-scope declaration and
//!   the two declaration-lane accessors that report it.
//! - `sharing_allocation_inventory.rs` — authoritative allocation kinds,
//!   locators, per-allocation observations, and region identities.
//! - `sharing_root_commitment.rs` — visibility commitments and the
//!   correctness-index posture.
//!
//! The observation itself:
//!
//! - `sharing_observation.rs` — the observation struct and its lane map.
//!
//! One file per truth-source lane of that observation, each owning the
//! accessors read from that one source:
//!
//! - `sharing_selection_metrics.rs` — the caller's own selection.
//! - `sharing_authoritative_byte_metrics.rs` — live owner byte totals.
//! - `sharing_authoritative_evidence_metrics.rs` — live owner identities and
//!   commitments behind those totals.
//! - `sharing_recorded_cost_metrics.rs` — counters written by earlier fork and
//!   publication work.
//! - `sharing_coordination_metrics.rs` — the selected branches' coordination
//!   cells.
//!
//! Assembly:
//!
//! - `sharing_scope.rs` — selection admission.
//! - `sharing_inventory.rs` — the owner walk over the admitted selection.
//! - `sharing_accounting.rs` — allocation deduplication and byte totals.
//! - `sharing_inspection.rs` — the entry point that assembles the observation.

#[path = "sharing_accounting.rs"]
pub(super) mod accounting;
#[path = "sharing_allocation_inventory.rs"]
mod allocation_inventory;
#[path = "sharing_authoritative_byte_metrics.rs"]
mod authoritative_byte_metrics;
#[path = "sharing_authoritative_evidence_metrics.rs"]
mod authoritative_evidence_metrics;
#[path = "sharing_byte_metric_scope.rs"]
mod byte_metric_scope;
#[path = "sharing_coordination_metrics.rs"]
mod coordination_metrics;
#[path = "sharing_inspection.rs"]
mod inspection;
#[path = "sharing_inspection_denial.rs"]
mod inspection_denial;
#[path = "sharing_inventory.rs"]
mod inventory;
#[path = "sharing_observation.rs"]
mod observation;
#[path = "sharing_recorded_cost_metrics.rs"]
mod recorded_cost_metrics;
#[path = "sharing_root_commitment.rs"]
mod root_commitment;
#[path = "sharing_scope.rs"]
mod scope;
#[path = "sharing_selection_metrics.rs"]
mod selection_metrics;

use accounting::RelationalAuthoritativeAllocationAccounting;

pub use allocation_inventory::{
    RelationalAuthoritativeAllocationKind, RelationalAuthoritativeAllocationLocator,
    RelationalAuthoritativeAllocationObservation, RelationalStorageRegionLocator,
};
pub use byte_metric_scope::{
    RelationalSharingByteMetricScope, RELATIONAL_SHARING_INSPECTION_VERSION,
};
pub use inspection_denial::RelationalBranchSharingInspectionDenial;
pub use observation::RelationalBranchSharingObservation;
pub use root_commitment::{
    RelationalCorrectnessIndexPosture, RelationalVisibilityCommitmentObservation,
};
