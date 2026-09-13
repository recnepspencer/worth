use crate::data::error::SignalError;
use crate::data::telemetry::CheckpointTelemetry;
use crate::logic::transaction::{
    CheckpointRecord, ReconstructabilityRecord, TemporalReconstructabilityArtifact,
};
use crate::state::{
    SignalCheckpointImage, SignalSnapshotId, SignalSnapshotV1, SnapshotArtifactRetentionPolicy,
    SnapshotRestoreIntent,
};

use super::{BranchState, SnapshotBranchState};

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Captures the canonical cell state without consulting runtime-global state.
    pub(crate) fn capture_for_owner_cell(
        &mut self,
        reserved_snapshot_id: SignalSnapshotId,
    ) -> Result<(SignalSnapshotV1, SnapshotBranchState<D, I, T>), SignalError> {
        self.graph_mut().interrupt_observation_at_boundary();
        let installed = self.graph().installed_runtime_policy();
        let request_metadata = installed.requested_policy();
        let artifact_retention =
            SnapshotArtifactRetentionPolicy::from_retention_budget(installed.retention_budget());
        let meta = self
            .graph_mut()
            .diagnostics_state_mut()
            .allocate_snapshot_meta_with_reserved_id(
                reserved_snapshot_id,
                request_metadata,
                artifact_retention,
            );
        crate::diagnostics::recorder::record_snapshot_event(
            self.graph_mut(),
            crate::diagnostics::replay::ReplayEventKind::SnapshotCaptured,
            Some(meta.snapshot_id),
            format!("snapshot {}", meta.snapshot_id.0),
        );
        let diagnostics = self
            .graph()
            .diagnostics_state()
            .snapshot_payload_with_retention(artifact_retention);
        let captures_telemetry = self.graph().captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        );
        let graph_telemetry = if captures_telemetry {
            *self.graph().telemetry()
        } else {
            Default::default()
        };
        let retained_replay = self
            .graph()
            .observe()
            .replay_events()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let snapshot_id = meta.snapshot_id;
        let replay_head = meta.replay_head;
        let mut diagnostic_graph = self.graph().clone_stateful();
        diagnostic_graph.clear_branch_mutation_nodes();
        let snapshot = SignalSnapshotV1 {
            meta,
            checkpoint_image: SignalCheckpointImage {
                authority: self.graph().capture_checkpoint_authority(),
                dependency_snapshot_batch: self
                    .graph()
                    .capture_checkpoint_dependency_snapshot_batch(),
                graph_telemetry,
            },
            diagnostic_graph,
            diagnostics,
            graph_telemetry,
            runtime_telemetry: captures_telemetry.then_some(*self.runtime_telemetry()),
            reconstructability: Some(ReconstructabilityRecord::from_snapshot_boundary(
                self.branch_id(),
                snapshot_id,
                replay_head,
                CheckpointRecord::from_checkpoint_telemetry(CheckpointTelemetry {
                    checkpoint_flushes: self.checkpoint().telemetry().checkpoint.checkpoint_flushes,
                    checkpoint_flush_nanos: self
                        .checkpoint()
                        .telemetry()
                        .checkpoint
                        .checkpoint_flush_nanos,
                    snapshot_restore_count: self
                        .runtime_telemetry()
                        .checkpoint
                        .snapshot_restore_count,
                    snapshot_restore_apply_active_policy_count: self
                        .runtime_telemetry()
                        .checkpoint
                        .snapshot_restore_apply_active_policy_count,
                    snapshot_restore_shared_delta_node_count: self
                        .runtime_telemetry()
                        .checkpoint
                        .snapshot_restore_shared_delta_node_count,
                    snapshot_restore_coarse_reason_count: self
                        .runtime_telemetry()
                        .checkpoint
                        .snapshot_restore_coarse_reason_count,
                    checkpoint_size: self.runtime_telemetry().checkpoint.checkpoint_size,
                    journal_replay_span: self.runtime_telemetry().checkpoint.journal_replay_span,
                    restore_authority_breadth: self
                        .runtime_telemetry()
                        .checkpoint
                        .restore_authority_breadth,
                    restore_required_derived_breadth: self
                        .runtime_telemetry()
                        .checkpoint
                        .restore_required_derived_breadth,
                    restore_diagnostic_richness_breadth: self
                        .runtime_telemetry()
                        .checkpoint
                        .restore_diagnostic_richness_breadth,
                    ..CheckpointTelemetry::default()
                }),
                &retained_replay,
                TemporalReconstructabilityArtifact::from_temporal_state(self.temporal()),
            )),
        };
        self.mutation_ledger_mut().clear_all(Some(snapshot_id));
        Ok((snapshot, SnapshotBranchState::from_branch_state(self)))
    }

    /// Reconstructs a replacement off to the side so denial never mutates the cell.
    pub(crate) fn prepare_owner_cell_restore(
        &self,
        snapshot_state: SnapshotBranchState<D, I, T>,
        snapshot: &SignalSnapshotV1,
    ) -> Result<Self, SignalError> {
        let mut graph = self.graph().clone_stateful();
        graph.restore_snapshot_with_intent(
            snapshot,
            SnapshotRestoreIntent::restore_runtime_truth(),
        )?;
        let runtime_telemetry = graph
            .captures_observation_surface(
                crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
            )
            .then_some(snapshot.runtime_telemetry)
            .flatten();
        let mut restored = snapshot_state.into_branch_state(graph, runtime_telemetry);
        if restored.branch_id() != self.branch_id() {
            return Err(SignalError::internal(
                "owner snapshot state changed branch identity during restore",
            ));
        }
        crate::diagnostics::recorder::record_snapshot_restore_lineage(
            restored.graph_mut(),
            snapshot.meta.snapshot_id,
        );
        Ok(restored)
    }
}
