use crate::data::graph::SignalGraph;
use crate::diagnostics::replay::{ReplayEvent, ReplayEventDetail, ReplayEventKind};
use crate::state::SignalSnapshotId;

pub(crate) fn record_transaction_semantic_event(
    graph: &mut SignalGraph,
    kind: ReplayEventKind,
    detail: impl Into<String>,
    execution_record_id: Option<u64>,
    semantic_segment_id: Option<u64>,
) -> crate::diagnostics::state::DiagnosticPublicationWork {
    if !graph.captures_observation_surface(
        crate::logic::transaction::SignalObservationSurface::ReplayDetail,
    ) {
        return Default::default();
    }
    let cursor = graph.diagnostics_state_mut().allocate_replay_cursor();
    let branch_id = graph.observe().current_branch().id;
    graph
        .diagnostics_state_mut()
        .record_replay_event(ReplayEvent::new(
            cursor,
            kind,
            branch_id,
            None,
            None,
            execution_record_id,
            semantic_segment_id,
            None,
            None,
            None,
            None,
            Some(ReplayEventDetail::Message(detail.into())),
        ))
}

pub(crate) fn record_snapshot_event(
    graph: &mut SignalGraph,
    kind: ReplayEventKind,
    snapshot_id: Option<SignalSnapshotId>,
    detail: impl Into<String>,
) {
    if !graph.captures_observation_surface(
        crate::logic::transaction::SignalObservationSurface::ReplayDetail,
    ) {
        return;
    }
    let cursor = graph.diagnostics_state_mut().allocate_replay_cursor();
    let branch_id = graph.observe().current_branch().id;
    graph
        .diagnostics_state_mut()
        .record_replay_event(ReplayEvent::new(
            cursor,
            kind,
            branch_id,
            snapshot_id,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(ReplayEventDetail::Message(detail.into())),
        ));
}
