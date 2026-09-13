use crate::data::error::SignalError;
use crate::state::{
    SignalBranchHandle, SignalCheckpointImage, SignalSnapshotV1, SnapshotArtifactRetentionPolicy,
};

use super::super::super::runtime_state::SignalRuntime;
use super::super::branches::SnapshotBranchState;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn capture_snapshot(&mut self) -> Result<SignalSnapshotV1, SignalError> {
        self.ensure_branch_snapshot_storage_available()?;
        Self::ensure_managed_queue_branch_transfer_allowed(&self.resource)?;
        let (next_snapshot_id, _) = self
            .graph
            .diagnostics_state()
            .branch_snapshot_allocator_state();
        self.branches
            .synchronize_snapshot_identity_high_water(next_snapshot_id);
        let snapshot_id = self
            .branches
            .reserve_snapshot_identity()
            .map_err(snapshot_identity_exhausted)?;
        let branch_id = self.graph.current_branch().id;
        let next_generation = self
            .branches
            .next_branch_head_generation(branch_id)
            .map_err(|denial| {
                SignalError::internal(format!(
                    "Signal snapshot generation cannot advance: {denial:?}"
                ))
            })?;
        let mut snapshot = self.graph.capture_snapshot_with_reserved_id(snapshot_id);
        let retained_replay = self
            .graph
            .observe()
            .replay_events()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        snapshot.diagnostic_graph.clear_branch_mutation_nodes();
        snapshot.runtime_telemetry = self
            .graph
            .captures_observation_surface(
                crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
            )
            .then_some(self.telemetry);
        snapshot.reconstructability = Some(
            super::super::super::reconstructability::ReconstructabilityRecord::from_snapshot_boundary(
                snapshot.meta.branch_id,
                snapshot.meta.snapshot_id,
                snapshot.meta.replay_head,
                super::super::super::reconstructability::CheckpointRecord::from_checkpoint_telemetry(
                    crate::data::telemetry::CheckpointTelemetry {
                        event_flushes: self.event_bus.telemetry().checkpoint.event_flushes,
                        event_flush_nanos: self.event_bus.telemetry().checkpoint.event_flush_nanos,
                        checkpoint_flushes: self
                            .checkpoint
                            .telemetry()
                            .checkpoint
                            .checkpoint_flushes,
                        checkpoint_flush_nanos: self
                            .checkpoint
                            .telemetry()
                            .checkpoint
                            .checkpoint_flush_nanos,
                        rollback_count: self.event_bus.telemetry().checkpoint.rollback_count,
                        snapshot_restore_count: self.telemetry.checkpoint.snapshot_restore_count,
                        snapshot_restore_apply_active_policy_count: self
                            .telemetry
                            .checkpoint
                            .snapshot_restore_apply_active_policy_count,
                        snapshot_restore_shared_delta_node_count: self
                            .telemetry
                            .checkpoint
                            .snapshot_restore_shared_delta_node_count,
                        snapshot_restore_coarse_reason_count: self
                            .telemetry
                            .checkpoint
                            .snapshot_restore_coarse_reason_count,
                        checkpoint_size: self.telemetry.checkpoint.checkpoint_size,
                        journal_replay_span: self.telemetry.checkpoint.journal_replay_span,
                        restore_authority_breadth: self
                            .telemetry
                            .checkpoint
                            .restore_authority_breadth,
                        restore_required_derived_breadth: self
                            .telemetry
                            .checkpoint
                            .restore_required_derived_breadth,
                        restore_diagnostic_richness_breadth: self
                            .telemetry
                            .checkpoint
                            .restore_diagnostic_richness_breadth,
                    },
                ),
                &retained_replay,
                super::super::super::reconstructability::TemporalReconstructabilityArtifact::from_temporal_state(
                    &self.temporal,
                ),
            ),
        );
        let mut branch_state = self.capture_heavy_branch_state()?;
        branch_state
            .mutation_ledger_mut()
            .clear_all(Some(snapshot.meta.snapshot_id));
        self.branches
            .set_branch_head_snapshot(branch_id, snapshot.meta.snapshot_id);
        self.project_branch_catalog();
        self.branches
            .project_catalog(branch_id, branch_state.graph_mut());
        self.branches
            .project_catalog(branch_id, &mut snapshot.diagnostic_graph);
        self.branches.insert_snapshot(
            SnapshotBranchState::from_branch_state(&branch_state).packet(snapshot.meta.snapshot_id),
        );
        self.branches.observe_active_branch_state(&branch_state);
        self.branches
            .commit_branch_head_generation(branch_id, next_generation);
        Ok(snapshot)
    }

    pub(crate) fn capture_branch_snapshot(
        &mut self,
        branch: SignalBranchHandle,
    ) -> Result<SignalSnapshotV1, SignalError> {
        if branch.id == self.graph.current_branch().id {
            return self.capture_snapshot();
        }
        self.ensure_branch_snapshot_storage_available()?;
        let stored_state = self
            .branches
            .branch_state(branch.id)
            .ok_or_else(|| SignalError::unknown_branch(Some(branch.id), branch.name.clone()))?;
        Self::ensure_managed_queue_branch_transfer_allowed(stored_state.resource())?;
        let (next_snapshot_id, _) = stored_state
            .graph()
            .diagnostics_state()
            .branch_snapshot_allocator_state();
        self.branches
            .synchronize_snapshot_identity_high_water(next_snapshot_id);
        let snapshot_id = self
            .branches
            .reserve_snapshot_identity()
            .map_err(snapshot_identity_exhausted)?;
        let next_generation = self
            .branches
            .next_branch_head_generation(branch.id)
            .map_err(|denial| {
                SignalError::internal(format!(
                    "Signal snapshot generation cannot advance: {denial:?}"
                ))
            })?;
        self.graph.interrupt_observation_at_boundary();
        let Some((mut snapshot, snapshot_state)) =
            self.branches.with_stored_branch_state_mut(branch.id, |state| {
            state.graph_mut().interrupt_observation_at_boundary();
            let installed = state.graph().installed_runtime_policy();
            let request_metadata = installed.requested_policy();
            let artifact_retention =
                SnapshotArtifactRetentionPolicy::from_retention_budget(installed.retention_budget());
            let meta = state
                .graph_mut()
                .diagnostics_state_mut()
                .allocate_snapshot_meta_with_reserved_id(
                    snapshot_id,
                    request_metadata,
                    artifact_retention,
                );
            crate::diagnostics::recorder::record_snapshot_event(
                state.graph_mut(),
                crate::diagnostics::replay::ReplayEventKind::SnapshotCaptured,
                Some(meta.snapshot_id),
                format!("snapshot {}", meta.snapshot_id.0),
            );
            let diagnostics = state
                .graph()
                .diagnostics_state()
                .snapshot_payload_with_retention(artifact_retention);
            let captures_telemetry = state.graph().captures_observation_surface(
                crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
            );
            let graph_telemetry = if captures_telemetry {
                *state.graph().telemetry()
            } else {
                Default::default()
            };
            let retained_replay = state
                .graph()
                .observe()
                .replay_events()
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            let replay_head = meta.replay_head;
            let snapshot_id = meta.snapshot_id;
            let snapshot = SignalSnapshotV1 {
                meta,
                    checkpoint_image: SignalCheckpointImage {
                        authority: state.graph().capture_checkpoint_authority(),
                        dependency_snapshot_batch: state
                            .graph()
                            .capture_checkpoint_dependency_snapshot_batch(),
                        graph_telemetry,
                    },
                    diagnostic_graph: {
                        let mut graph = state.graph().clone_stateful();
                        graph.clear_branch_mutation_nodes();
                        graph
                    },
                    diagnostics,
                graph_telemetry,
                runtime_telemetry: captures_telemetry.then_some(*state.runtime_telemetry()),
                reconstructability: Some(
                    super::super::super::reconstructability::ReconstructabilityRecord::from_snapshot_boundary(
                        branch.id,
                        snapshot_id,
                        replay_head,
                        super::super::super::reconstructability::CheckpointRecord::from_checkpoint_telemetry(
                            crate::data::telemetry::CheckpointTelemetry {
                                event_flushes: 0,
                                event_flush_nanos: 0,
                                checkpoint_flushes: state
                                    .checkpoint()
                                    .telemetry()
                                    .checkpoint
                                    .checkpoint_flushes,
                                checkpoint_flush_nanos: state
                                    .checkpoint()
                                    .telemetry()
                                    .checkpoint
                                    .checkpoint_flush_nanos,
                                rollback_count: 0,
                                snapshot_restore_count: state
                                    .runtime_telemetry()
                                    .checkpoint
                                    .snapshot_restore_count,
                                snapshot_restore_apply_active_policy_count: state
                                    .runtime_telemetry()
                                    .checkpoint
                                    .snapshot_restore_apply_active_policy_count,
                                snapshot_restore_shared_delta_node_count: state
                                    .runtime_telemetry()
                                    .checkpoint
                                    .snapshot_restore_shared_delta_node_count,
                                snapshot_restore_coarse_reason_count: state
                                    .runtime_telemetry()
                                    .checkpoint
                                    .snapshot_restore_coarse_reason_count,
                                checkpoint_size: state.runtime_telemetry().checkpoint.checkpoint_size,
                                journal_replay_span: state
                                    .runtime_telemetry()
                                    .checkpoint
                                    .journal_replay_span,
                                restore_authority_breadth: state
                                    .runtime_telemetry()
                                    .checkpoint
                                    .restore_authority_breadth,
                                restore_required_derived_breadth: state
                                    .runtime_telemetry()
                                    .checkpoint
                                    .restore_required_derived_breadth,
                                restore_diagnostic_richness_breadth: state
                                    .runtime_telemetry()
                                    .checkpoint
                                    .restore_diagnostic_richness_breadth,
                            },
                        ),
                        &retained_replay,
                        super::super::super::reconstructability::TemporalReconstructabilityArtifact::from_temporal_state(
                            state.temporal(),
                        ),
                    ),
                ),
            };
            state
                .mutation_ledger_mut()
                .clear_all(Some(snapshot.meta.snapshot_id));
            (snapshot, SnapshotBranchState::from_branch_state(state))
        }) else {
            return Err(SignalError::unknown_branch(Some(branch.id), branch.name));
        };
        self.branches
            .insert_snapshot(snapshot_state.packet(snapshot.meta.snapshot_id));
        self.branches
            .set_branch_head_snapshot(branch.id, snapshot.meta.snapshot_id);
        self.branches
            .project_catalog(branch.id, &mut snapshot.diagnostic_graph);
        self.project_branch_catalog();
        self.branches
            .commit_branch_head_generation(branch.id, next_generation);
        Ok(snapshot)
    }

    fn ensure_branch_snapshot_storage_available(&self) -> Result<(), SignalError> {
        self.branches
            .ensure_snapshot_storage_available()
            .map_err(|denial| {
                SignalError::invalid_input(format!(
                    "Signal branch snapshot storage exhausted before movement: {denial:?}"
                ))
            })
    }
}

fn snapshot_identity_exhausted(snapshot_id: crate::state::SignalSnapshotId) -> SignalError {
    SignalError::invalid_input(format!(
        "Signal snapshot identity exhausted at {} before movement",
        snapshot_id.0
    ))
}
