use crate::data::graph::SignalGraph;
use crate::diagnostics::lineage::LineageRecord;

use crate::state::SignalBranchId;

pub(crate) fn record_branch_fork_lineage(
    graph: &mut SignalGraph,
    created_branch_id: SignalBranchId,
    parent_branch_id: SignalBranchId,
    created_branch_display_name: impl Into<String>,
    parent_branch_display_name: impl Into<String>,
) {
    if !graph.captures_observation_surface(
        crate::logic::transaction::SignalObservationSurface::DescriptiveLineage,
    ) {
        return;
    }
    let sequence = graph.diagnostics_state_mut().allocate_lineage_sequence();
    let emitted_on_branch_id = graph.observe().current_branch().id;
    graph
        .diagnostics_state_mut()
        .record_lineage_record(LineageRecord::branch_fork(
            sequence,
            emitted_on_branch_id,
            created_branch_id,
            parent_branch_id,
            created_branch_display_name,
            parent_branch_display_name,
        ));
}

pub(crate) fn record_branch_switch_lineage(
    graph: &mut SignalGraph,
    from_branch_id: SignalBranchId,
    to_branch_id: SignalBranchId,
    from_branch_display_name: impl Into<String>,
    to_branch_display_name: impl Into<String>,
) {
    if !graph.captures_observation_surface(
        crate::logic::transaction::SignalObservationSurface::DescriptiveLineage,
    ) {
        return;
    }
    let sequence = graph.diagnostics_state_mut().allocate_lineage_sequence();
    let emitted_on_branch_id = graph.observe().current_branch().id;
    graph
        .diagnostics_state_mut()
        .record_lineage_record(LineageRecord::branch_switch(
            sequence,
            emitted_on_branch_id,
            from_branch_id,
            to_branch_id,
            from_branch_display_name,
            to_branch_display_name,
        ));
}
