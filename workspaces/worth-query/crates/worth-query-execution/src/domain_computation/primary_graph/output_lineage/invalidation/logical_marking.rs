//! Logical delivery diagnostics. These values carry no currentness authority.

use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::history::CommitId;

use super::super::RecordedSettlementIdentity;

mod read_observation;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct LogicalMarkingCounts {
    pub(super) posting_key_lookups: u64,
    // Includes repeated matches of one ordinal through different posting keys.
    pub(super) matched_fact_postings: u64,
    pub(super) marked_fact_ordinals: u64,
    // Includes directly marked roots and equality predecessors as well as
    // downstream vertices; the worklist visits each exact identity once.
    pub(super) visited_vertices: u64,
    pub(super) downstream_edges: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NativeMarkingPrecision {
    Exact(LogicalMarkingCounts),
    DeclaredChangeUnavailable,
    /// Matched fan-out exceeded the installed marking ceiling. The commit
    /// published as a discontinuity; the counts are those spent before it.
    MarkingCeilingExceeded(LogicalMarkingCounts),
    /// The marked version exceeded the installed retained capacity. The commit
    /// published as a discontinuity; the counts are those of the full marking.
    RetainedCapacityExhausted(LogicalMarkingCounts),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NativeMarkingReport {
    pub(super) commit: CommitId,
    pub(super) precision: NativeMarkingPrecision,
}

pub(super) struct AppliedNativeMarking {
    pub(super) affected: OrdSet<Arc<RecordedSettlementIdentity>>,
    pub(super) report: NativeMarkingReport,
}
