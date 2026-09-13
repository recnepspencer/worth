mod capture;
mod restore_selection;
mod validation;

use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::diagnostics::policy::OrdinaryAccessLane;
use crate::diagnostics::summary::{ExecutionHistorySummary, GraphSummary};
use crate::state::{
    SignalBranchHandle, SignalSnapshotV1, SnapshotArtifactRestoreMode,
    SnapshotDependencyRestoreMode, SnapshotRestoreIntent,
};

use super::super::runtime_state::SignalRuntime;
use super::branches::SnapshotBranchState;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn restore_snapshot(
        &mut self,
        snapshot: &SignalSnapshotV1,
    ) -> Result<(), SignalError> {
        self.restore_snapshot_with_intent(snapshot, SnapshotRestoreIntent::restore_runtime_truth())
    }

    pub(crate) fn restore_snapshot_with_intent(
        &mut self,
        snapshot: &SignalSnapshotV1,
        intent: SnapshotRestoreIntent,
    ) -> Result<(), SignalError> {
        let restored_branch_id = snapshot.meta.branch_id;
        let next_generation = self
            .branches
            .next_branch_head_generation(restored_branch_id)
            .map_err(|denial| {
                SignalError::internal(format!(
                    "Signal restore generation cannot advance: {denial:?}"
                ))
            })?;
        let reconstructability_proof = snapshot.reconstructability_proof()?;
        let restore_plan = self.plan_snapshot_restore_for_target(snapshot, intent)?;
        self.graph.validate_snapshot_compatibility(snapshot)?;
        if matches!(
            intent.dependency_state,
            SnapshotDependencyRestoreMode::SeedRecomputationFromSnapshot
        ) {
            return Err(SignalError::invalid_input(
                "snapshot restore intent `SeedRecomputationFromSnapshot` is not implemented yet",
            ));
        }
        let snapshot_state =
            self.load_snapshot_branch_state(snapshot.meta.branch_id, snapshot.meta.snapshot_id)?;
        Self::ensure_managed_queue_branch_transfer_allowed(&self.resource)?;
        if let Some(snapshot_state) = snapshot_state.as_ref() {
            Self::ensure_managed_queue_branch_transfer_allowed(snapshot_state.resource())?;
        }
        if let Some(snapshot_state) = snapshot_state {
            let (current_diagnostics, target_policy) =
                self.snapshot_restore_target_context(restored_branch_id)?;
            self.restore_snapshot_branch_state(
                snapshot,
                intent,
                &reconstructability_proof,
                &restore_plan,
                snapshot_state,
                &current_diagnostics,
                target_policy,
            )?;
            self.branches.commit_branch_restore_generation(
                restored_branch_id,
                next_generation,
                snapshot.meta.snapshot_id,
            );
            return Ok(());
        }

        self.graph.restore_snapshot_with_intent(snapshot, intent)?;
        if self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        ) {
            if let Some(telemetry) = &snapshot.runtime_telemetry {
                self.telemetry = *telemetry;
            }
        }
        self.branches
            .set_branch_head_snapshot(restored_branch_id, snapshot.meta.snapshot_id);
        self.project_branch_catalog();
        self.branches.commit_branch_restore_generation(
            restored_branch_id,
            next_generation,
            snapshot.meta.snapshot_id,
        );
        Ok(())
    }

    pub(crate) fn restore_branch_snapshot(
        &mut self,
        branch: SignalBranchHandle,
        snapshot: &SignalSnapshotV1,
    ) -> Result<(), SignalError> {
        self.restore_branch_snapshot_with_intent(
            branch,
            snapshot,
            SnapshotRestoreIntent::restore_runtime_truth(),
        )
    }

    pub(crate) fn restore_branch_snapshot_with_intent(
        &mut self,
        branch: SignalBranchHandle,
        snapshot: &SignalSnapshotV1,
        intent: SnapshotRestoreIntent,
    ) -> Result<(), SignalError> {
        let reconstructability_proof = snapshot.reconstructability_proof()?;
        if snapshot.meta.branch_id != branch.id {
            return Err(SignalError::incompatible_snapshot(format!(
                "snapshot `{}` from branch `{}` cannot be restored into branch `{}`",
                snapshot.meta.snapshot_id.0, snapshot.meta.branch_name, branch.name
            )));
        }
        let restore_plan = self.plan_snapshot_restore_for_target(snapshot, intent)?;
        self.graph.validate_snapshot_compatibility(snapshot)?;
        if branch.id == self.graph.current_branch().id {
            return self.restore_snapshot_with_intent(snapshot, intent);
        }
        let next_generation = self
            .branches
            .next_branch_head_generation(branch.id)
            .map_err(|denial| {
                SignalError::internal(format!(
                    "Signal restore generation cannot advance: {denial:?}"
                ))
            })?;
        if matches!(
            intent.dependency_state,
            SnapshotDependencyRestoreMode::SeedRecomputationFromSnapshot
        ) {
            return Err(SignalError::invalid_input(
                "snapshot restore intent `SeedRecomputationFromSnapshot` is not implemented yet",
            ));
        }
        let (snapshot_state, current_diagnostics, target_policy) =
            self.load_noncurrent_restore_inputs(&branch, snapshot)?;
        let graph = self.restore_snapshot_graph(
            snapshot,
            &reconstructability_proof,
            &restore_plan,
            &current_diagnostics,
            intent,
            target_policy,
        )?;
        let mut state = Self::finalize_restored_branch_state(snapshot_state, graph, snapshot);
        self.record_snapshot_restore_telemetry(intent, &restore_plan);
        self.graph.interrupt_observation_at_boundary();
        self.branches
            .set_branch_head_snapshot(branch.id, snapshot.meta.snapshot_id);
        self.branches.project_catalog(branch.id, state.graph_mut());
        self.branches.store_branch_state(state);
        self.project_branch_catalog();
        self.branches.commit_branch_restore_generation(
            branch.id,
            next_generation,
            snapshot.meta.snapshot_id,
        );
        Ok(())
    }

    fn load_noncurrent_restore_inputs(
        &self,
        branch: &SignalBranchHandle,
        snapshot: &SignalSnapshotV1,
    ) -> Result<
        (
            SnapshotBranchState<D, I, T>,
            crate::diagnostics::state::DiagnosticsState,
            crate::runtime_policy::InstalledSignalRuntimePolicy,
        ),
        SignalError,
    > {
        let snapshot_state = self
            .branches
            .snapshot_state(snapshot.meta.branch_id, snapshot.meta.snapshot_id)
            .cloned()
            .ok_or_else(|| {
                SignalError::internal(format!(
                    "snapshot `{}:{}` is missing runtime-local branch semantic state",
                    snapshot.meta.branch_id.0, snapshot.meta.snapshot_id.0
                ))
            })?;
        let target_state = self
            .branches
            .branch_state(branch.id)
            .ok_or_else(|| SignalError::unknown_branch(Some(branch.id), branch.name.clone()))?;
        Self::ensure_managed_queue_branch_transfer_allowed(target_state.resource())?;
        Self::ensure_managed_queue_branch_transfer_allowed(snapshot_state.resource())?;
        Ok((
            snapshot_state,
            target_state.graph().diagnostics_state().clone(),
            target_state.graph().installed_runtime_policy(),
        ))
    }

    fn restore_snapshot_branch_state(
        &mut self,
        snapshot: &SignalSnapshotV1,
        intent: SnapshotRestoreIntent,
        reconstructability_proof: &crate::logic::transaction::ReconstructabilityProof,
        restore_plan: &crate::state::SnapshotRestorePlan,
        snapshot_state: SnapshotBranchState<D, I, T>,
        current_diagnostics: &crate::diagnostics::state::DiagnosticsState,
        target_policy: crate::runtime_policy::InstalledSignalRuntimePolicy,
    ) -> Result<(), SignalError> {
        let graph = self.restore_snapshot_graph(
            snapshot,
            reconstructability_proof,
            restore_plan,
            current_diagnostics,
            intent,
            target_policy,
        )?;
        self.branches
            .set_branch_head_snapshot(snapshot.meta.branch_id, snapshot.meta.snapshot_id);
        let mut state = Self::finalize_restored_branch_state(snapshot_state, graph, snapshot);
        self.branches
            .project_catalog(snapshot.meta.branch_id, state.graph_mut());
        let preserved_transaction = self.telemetry_snapshot().transaction;
        let interrupted_observation = self.graph.interrupt_observation_at_boundary();
        self.install_snapshot_restore_selection(snapshot.meta.branch_id, state)?;
        self.with_telemetry(|telemetry| {
            Self::merge_global_transaction_telemetry(
                preserved_transaction,
                &mut telemetry.transaction,
            );
        });
        if interrupted_observation {
            self.graph.record_boundary_interruption();
        }
        self.record_snapshot_restore_telemetry(intent, restore_plan);
        self.project_branch_catalog();
        Ok(())
    }

    fn record_snapshot_restore_telemetry(
        &mut self,
        intent: SnapshotRestoreIntent,
        restore_plan: &crate::state::SnapshotRestorePlan,
    ) {
        self.with_telemetry(|telemetry| {
            telemetry.checkpoint.snapshot_restore_count += 1;
            if matches!(
                intent.artifacts,
                SnapshotArtifactRestoreMode::ApplyActiveRuntimePolicy
            ) {
                telemetry
                    .checkpoint
                    .snapshot_restore_apply_active_policy_count += 1;
            }
            telemetry
                .checkpoint
                .snapshot_restore_shared_delta_node_count +=
                restore_plan.dependency_snapshot_delta_node_count();
            telemetry.checkpoint.snapshot_restore_coarse_reason_count +=
                restore_plan.coarse_reasons().len() as u64;
        });
    }

    fn restore_snapshot_graph(
        &self,
        snapshot: &SignalSnapshotV1,
        reconstructability_proof: &crate::logic::transaction::ReconstructabilityProof,
        restore_plan: &crate::state::SnapshotRestorePlan,
        current_diagnostics: &crate::diagnostics::state::DiagnosticsState,
        intent: SnapshotRestoreIntent,
        active_policy: crate::runtime_policy::InstalledSignalRuntimePolicy,
    ) -> Result<SignalGraph, SignalError> {
        let mut graph =
            self.restore_runtime_authority_from_snapshot_proof(snapshot, reconstructability_proof)?;
        if matches!(
            intent.artifacts,
            SnapshotArtifactRestoreMode::ApplyActiveRuntimePolicy
        ) {
            graph.install_compiled_runtime_policy(active_policy.requested_policy(), active_policy);
        }
        if let Some(mut telemetry) = graph.telemetry_mut() {
            *telemetry = snapshot.checkpoint_image.graph_telemetry;
        }
        Self::rebuild_runtime_required_derived_from_proof(
            &mut graph,
            snapshot,
            reconstructability_proof,
            restore_plan,
        )?;
        Self::apply_runtime_diagnostic_policy_richness(
            &mut graph,
            snapshot,
            current_diagnostics,
            intent,
        );
        graph.with_telemetry(|telemetry| telemetry.checkpoint.snapshot_restore_count += 1);
        if matches!(
            intent.artifacts,
            SnapshotArtifactRestoreMode::ApplyActiveRuntimePolicy
        ) {
            graph.with_telemetry(|telemetry| {
                telemetry
                    .checkpoint
                    .snapshot_restore_apply_active_policy_count += 1;
            });
        }
        Ok(graph)
    }

    fn finalize_restored_branch_state(
        snapshot_state: SnapshotBranchState<D, I, T>,
        graph: SignalGraph,
        snapshot: &SignalSnapshotV1,
    ) -> super::branches::BranchState<D, I, T> {
        let restore_optional_telemetry = graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        );
        let runtime_telemetry = restore_optional_telemetry
            .then_some(snapshot.runtime_telemetry)
            .flatten();
        let mut state = snapshot_state.into_branch_state(graph, runtime_telemetry);
        crate::diagnostics::recorder::record_snapshot_restore_lineage(
            state.graph_mut(),
            snapshot.meta.snapshot_id,
        );
        let retention_budget = state.graph().installed_runtime_policy().retention_budget();
        let profile = state.graph().diagnostics_profile();
        let history = ExecutionHistorySummary::from_graph(
            state.graph(),
            profile,
            retention_budget.detail_limit,
            retention_budget.retain_history_details,
            OrdinaryAccessLane,
        );
        let graph_summary = GraphSummary::from_graph(
            state.graph(),
            profile,
            retention_budget.detail_limit,
            OrdinaryAccessLane,
        );
        state
            .graph_mut()
            .diagnostics_state_mut()
            .refresh_retained_views(history, graph_summary);
        state
    }
}
