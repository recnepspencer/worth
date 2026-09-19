use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::OutputChange;
use crate::data::reuse::ReuseOrigin;
use crate::data::trace::RuntimeArtifactFinalizeImage;
use crate::diagnostics::lineage::{ArtifactTransitionKind, InvalidationCause, LineageRecord};
use crate::logic::planner::{ExecutionRecordId, SemanticSegmentId};

fn derive_lineage_transition(
    graph: &mut SignalGraph,
    before_trace: Option<&RuntimeArtifactFinalizeImage>,
    after_trace: &RuntimeArtifactFinalizeImage,
) -> (
    crate::diagnostics::lineage::LineageArtifactId,
    Option<crate::diagnostics::lineage::LineageArtifactId>,
    ArtifactTransitionKind,
) {
    let previous_artifact_id = before_trace.and_then(|summary| summary.lineage_artifact_id().get());
    let (artifact_id, transition) = if matches!(
        after_trace.reuse_origin(),
        ReuseOrigin::MemoizedArtifactReuse
            | ReuseOrigin::SnapshotRestore
            | ReuseOrigin::ReconciliationAdoption
            | ReuseOrigin::CrossIdentityPersistentReuse
            | ReuseOrigin::PartialArtifactSplice
    ) {
        let artifact_id = previous_artifact_id
            .unwrap_or_else(|| graph.diagnostics_state_mut().allocate_lineage_artifact_id());
        (
            artifact_id,
            match after_trace.reuse_origin() {
                ReuseOrigin::MemoizedArtifactReuse => ArtifactTransitionKind::MemoizedReuse,
                ReuseOrigin::SnapshotRestore => ArtifactTransitionKind::SnapshotRestoreReuse,
                ReuseOrigin::ReconciliationAdoption => {
                    ArtifactTransitionKind::ReconciliationAdoption
                }
                ReuseOrigin::CrossIdentityPersistentReuse => {
                    ArtifactTransitionKind::CrossIdentityPersistentReuse {
                        correspondence_kind: after_trace
                            .reuse_boundary_authority()
                            .and_then(|authority| authority.persistent_correspondence_kind())
                            .unwrap_or(crate::data::reuse::PersistentCorrespondenceKind::Unknown),
                    }
                }
                ReuseOrigin::PartialArtifactSplice => {
                    ArtifactTransitionKind::PartialArtifactSplice {
                        composition_region_count: after_trace
                            .reuse_boundary_authority()
                            .map(|authority| authority.composition_region_count())
                            .unwrap_or(0),
                        recomputed_region_count: after_trace.changed_partition_count(),
                    }
                }
                ReuseOrigin::FreshCompute | ReuseOrigin::OutputSuppressed => {
                    unreachable!("guarded by matches!")
                }
            },
        )
    } else if previous_artifact_id.is_some()
        && matches!(
            after_trace.output_change(),
            OutputChange::Refreshed | OutputChange::Unchanged
        )
    {
        (
            previous_artifact_id.expect("checked above"),
            ArtifactTransitionKind::Refreshed {
                output_change: after_trace.output_change(),
            },
        )
    } else {
        (
            graph.diagnostics_state_mut().allocate_lineage_artifact_id(),
            ArtifactTransitionKind::Replaced,
        )
    };
    (artifact_id, previous_artifact_id, transition)
}

#[cfg(test)]
pub(crate) fn record_lineage_transition(
    graph: &mut SignalGraph,
    node: NodeId,
    before_trace: Option<&RuntimeArtifactFinalizeImage>,
    execution_record_id: ExecutionRecordId,
    semantic_segment_id: SemanticSegmentId,
) -> Result<(), crate::data::error::SignalError> {
    let Some(after_finalize_image) = graph.node_runtime_artifact_finalize_image(node)? else {
        return Ok(());
    };
    stamp_trace_summary_and_record_lineage_transition_from_image(
        graph,
        node,
        before_trace,
        &after_finalize_image,
        execution_record_id,
        semantic_segment_id,
        &mut crate::logic::evaluation::EvaluationWork::Ordinary,
    )
}

pub(crate) fn stamp_trace_summary_and_record_lineage_transition_from_image(
    graph: &mut SignalGraph,
    node: NodeId,
    before_trace: Option<&RuntimeArtifactFinalizeImage>,
    after_finalize_image: &RuntimeArtifactFinalizeImage,
    execution_record_id: ExecutionRecordId,
    semantic_segment_id: SemanticSegmentId,
    work: &mut crate::logic::evaluation::EvaluationWork<'_>,
) -> Result<(), crate::data::error::SignalError> {
    let (artifact_id, previous_artifact_id, transition) =
        derive_lineage_transition(graph, before_trace, after_finalize_image);
    graph.stamp_runtime_artifact_lineage_and_execution(
        node,
        artifact_id,
        execution_record_id,
        semantic_segment_id,
        work,
    )?;
    if !graph.captures_observation_surface(
        crate::logic::transaction::SignalObservationSurface::DescriptiveLineage,
    ) {
        return Ok(());
    }
    let sequence = graph.diagnostics_state_mut().allocate_lineage_sequence();
    let emitted_on_branch_id = graph.observe().current_branch().id;
    graph.record_evaluation_lineage(
        LineageRecord::artifact_transition(
            sequence,
            emitted_on_branch_id,
            node,
            artifact_id,
            previous_artifact_id,
            execution_record_id,
            semantic_segment_id,
            transition,
        ),
        work,
    )?;
    Ok(())
}

pub(crate) fn record_invalidation_lineage(
    graph: &mut SignalGraph,
    node: NodeId,
    cause: InvalidationCause,
) -> Result<(), crate::data::error::SignalError> {
    if !graph.captures_observation_surface(
        crate::logic::transaction::SignalObservationSurface::DescriptiveLineage,
    ) {
        return Ok(());
    }
    let Some(artifact_id) = graph.node_lineage_artifact_id(node).ok().flatten() else {
        return Ok(());
    };
    let sequence = graph.diagnostics_state_mut().allocate_lineage_sequence();
    let emitted_on_branch_id = graph.observe().current_branch().id;
    graph.record_evaluation_lineage(
        LineageRecord::invalidation(sequence, emitted_on_branch_id, node, artifact_id, cause),
        &mut crate::logic::evaluation::EvaluationWork::Ordinary,
    )
}
