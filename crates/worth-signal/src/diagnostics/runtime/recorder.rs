mod artifacts;
mod branching;
mod events;
mod failure;
mod snapshots;

use crate::data::graph::SignalGraph;

pub(crate) struct DiagnosticsRecorder<'a> {
    graph: &'a mut SignalGraph,
}

impl<'a> DiagnosticsRecorder<'a> {
    pub(crate) fn new(graph: &'a mut SignalGraph) -> Self {
        Self { graph }
    }
}

#[cfg(test)]
pub(crate) use artifacts::record_lineage_transition;
pub(crate) use artifacts::{
    derive_lineage_transition, record_invalidation_lineage,
    stamp_trace_summary_and_record_lineage_transition_from_image,
};
pub(crate) use branching::{
    record_branch_fork_lineage, record_branch_merge_failure, record_branch_merge_summary,
    record_branch_switch_lineage,
};
pub(crate) use events::{record_snapshot_event, record_transaction_semantic_event};
pub(crate) use snapshots::record_snapshot_restore_lineage;
