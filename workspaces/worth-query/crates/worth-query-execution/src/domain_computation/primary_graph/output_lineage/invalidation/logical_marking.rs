//! Logical delivery diagnostics. These values carry no currentness authority.

use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::history::CommitId;

use super::super::RecordedSettlementIdentity;

#[cfg(test)]
mod read_observation;

/// Native matching and closure diagnostics. Posting visits precede filtering;
/// marked ordinals and vertices are distinct identities, not admission units.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryNativeMarkingWork {
    pub posting_key_lookups: u64,
    // Includes repeated matches through different keys and genuinely discovered
    // queued pairs when a later stop precedes marking. No pair is counted twice
    // within one key, and unchanged physical sharing never erases logical fanout.
    pub matched_fact_postings: u64,
    pub marked_fact_ordinals: u64,
    // Includes directly marked roots and equality predecessors as well as
    // downstream vertices; the worklist visits each exact identity once.
    pub visited_vertices: u64,
    pub downstream_edges: u64,
}

/// Precision reached by native delivery preparation. This descriptive result
/// alone proves neither publication acceptance nor source currentness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryNativeMarkingPrecision {
    Exact(WorthQueryNativeMarkingWork),
    DeclaredChangeUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NativeMarkingReport {
    pub(super) commit: CommitId,
    pub(super) precision: WorthQueryNativeMarkingPrecision,
}

pub(super) struct AppliedNativeMarking {
    pub(super) affected: OrdSet<Arc<RecordedSettlementIdentity>>,
    pub(super) report: NativeMarkingReport,
}
