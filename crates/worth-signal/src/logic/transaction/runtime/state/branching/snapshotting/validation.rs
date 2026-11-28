use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::state::{SignalSnapshotV1, SnapshotArtifactRestoreMode, SnapshotRestoreIntent};

use super::super::super::runtime_state::SignalRuntime;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn restore_runtime_authority_from_snapshot_proof(
        &self,
        snapshot: &SignalSnapshotV1,
        proof: &crate::logic::transaction::ReconstructabilityProof,
    ) -> Result<SignalGraph, SignalError> {
        if proof.checkpoint.authority_branch_id != snapshot.meta.branch_id
            || proof.checkpoint.authority_snapshot_id != Some(snapshot.meta.snapshot_id)
        {
            return Err(SignalError::incompatible_snapshot(format!(
                "snapshot `{}` reconstructability proof does not match snapshot identity",
                snapshot.meta.snapshot_id.0
            )));
        }
        let mut graph =
            SignalGraph::restore_from_checkpoint_authority(&snapshot.checkpoint_image.authority)?;
        let active_nodes = graph.active_node_count() as u64;
        graph.with_telemetry(|telemetry| {
            telemetry.checkpoint.restore_authority_breadth += active_nodes
        });
        Ok(graph)
    }

    pub(super) fn rebuild_runtime_required_derived_from_proof(
        graph: &mut SignalGraph,
        snapshot: &SignalSnapshotV1,
        proof: &crate::logic::transaction::ReconstructabilityProof,
        restore_plan: &crate::state::SnapshotRestorePlan,
    ) -> Result<(), SignalError> {
        let mut rebuild_breadth = 0_u64;
        for requirement in &proof.required_rebuild {
            match requirement {
                crate::logic::transaction::RequiredDerivedRebuildSet::DependencyIndexes(_) => {
                    let classified_checkpoint_batch =
                        restore_plan.checkpoint_restore_batch().clone_inner();
                    rebuild_breadth +=
                        classified_checkpoint_batch.target_nodes().as_slice().len() as u64;
                    graph.apply_classified_snapshot_batch_commit(classified_checkpoint_batch)?;
                }
                crate::logic::transaction::RequiredDerivedRebuildSet::ReplaySuffix(replay) => {
                    if snapshot.diagnostics.replay_frames.len() < replay.replay_event_count as usize
                    {
                        return Err(SignalError::incompatible_snapshot(format!(
                            "snapshot `{}` replay payload is shorter than reconstructability proof",
                            snapshot.meta.snapshot_id.0
                        )));
                    }
                    rebuild_breadth += replay.replay_event_count as u64;
                }
                crate::logic::transaction::RequiredDerivedRebuildSet::TemporalState(temporal) => {
                    rebuild_breadth += temporal
                        .scheduled_wake_count
                        .saturating_add(temporal.ready_wake_count)
                        .saturating_add(temporal.retired_wake_count);
                }
            }
        }
        graph.with_telemetry(|telemetry| {
            telemetry.checkpoint.restore_required_derived_breadth += rebuild_breadth;
        });
        // Snapshot reconstruction must not publish the replayed changes again.
        if proof.journal.replay_event_count > 0 {
            graph.clear_branch_mutation_nodes();
        }
        graph.readmit_checkpoint_causes()?;
        Ok(())
    }

    pub(super) fn apply_runtime_diagnostic_policy_richness(
        graph: &mut SignalGraph,
        snapshot: &SignalSnapshotV1,
        current_diagnostics: &crate::diagnostics::state::DiagnosticsState,
        intent: SnapshotRestoreIntent,
    ) {
        graph
            .diagnostics_state_mut()
            .restore_snapshot_payload_preserving_history_from(
                snapshot.diagnostics.clone(),
                current_diagnostics,
            );
        if matches!(
            intent.artifacts,
            SnapshotArtifactRestoreMode::ApplyActiveRuntimePolicy
        ) {
            let installed = graph.installed_runtime_policy();
            graph
                .diagnostics_state_mut()
                .set_installed_policy(installed);
        }
        let diagnostics_breadth = snapshot.diagnostics.recent_history.len() as u64
            + snapshot.diagnostics.replay_frames.len() as u64
            + snapshot.diagnostics.explanation_facts.len() as u64
            + snapshot.diagnostics.provenance_facts.len() as u64
            + snapshot.diagnostics.lineage_records.len() as u64;
        graph.with_telemetry(|telemetry| {
            telemetry.checkpoint.restore_diagnostic_richness_breadth += diagnostics_breadth;
        });
    }
}
