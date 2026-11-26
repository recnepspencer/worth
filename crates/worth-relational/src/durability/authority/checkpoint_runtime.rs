//! Select and publish checkpoints for one actual native runtime.
use super::checkpointing::checkpoint_admission_error;
use super::diagnostics::checkpoint_created;
use crate::diagnostics::data::{DiagnosticsArtifactKind, DiagnosticsScope};
use crate::durability::authority::runtime_access::DurabilityAuthority;
use crate::durability::data::{
    DurabilityError, DurableCheckpoint, RelationalNativeCheckpointCaptureDenial as CaptureDenial,
};

impl<'runtime> DurabilityAuthority<'runtime> {
    pub fn native_checkpoint_with_policy(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        artifact_policy: crate::durability::data::RelationalCheckpointArtifactPolicy,
    ) -> Result<
        crate::durability::data::RelationalNativeCheckpoint,
        crate::durability::data::RelationalNativeCheckpointCaptureDenial,
    > {
        policy.check_live()?;
        crate::execution::check_request(execution)
            .map_err(crate::durability::data::RelationalNativeCheckpointCaptureDenial::Request)?;
        let basis = self.capture_checkpoint_basis(false)?;
        self.encode_native_capture(basis, execution, policy, artifact_policy)
    }

    pub(in crate::durability::authority) fn encode_native_capture(
        &self,
        basis: super::checkpoint_capture::CapturedCheckpointBasis,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        artifact_policy: crate::durability::data::RelationalCheckpointArtifactPolicy,
    ) -> Result<crate::durability::data::RelationalNativeCheckpoint, CaptureDenial> {
        let (checkpoint, transitions) = super::checkpointing::encode_native_capture(
            basis,
            execution,
            policy,
            artifact_policy,
            self.runtime.durability.checkpoint_selection(),
            self.runtime.durability.cold_source(),
            self.runtime.durability.medium(),
        )?;
        if self.runtime.durability.medium().is_some() {
            self.runtime
                .history
                .compact_branch_transitions_through(transitions);
        }
        Ok(checkpoint)
    }

    pub fn checkpoint(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DurableCheckpoint, crate::durability::data::RelationalRecoveryStop> {
        execution.check_live().map_err(|stop| {
            crate::durability::data::RelationalRecoveryStop::Request(stop.into())
        })?;
        let captured = self.capture_checkpoint_basis(true)?.demand_memory_image(
            &self.runtime.services.symbols,
            &self.runtime.lineage,
            execution,
        )?;
        self.finalize_captured_checkpoint(captured, execution)
    }

    /// `whole` selects every commit (the expanded memory image); a native
    /// capture selects resident commits and carries the cold prefix (`G7`).
    pub(crate) fn capture_checkpoint_basis(
        &self,
        whole: bool,
    ) -> Result<super::checkpoint_capture::CapturedCheckpointBasis, DurabilityError> {
        let checkpoint_routes = self.runtime.history.canonical_checkpoint_gate();
        let selection = checkpoint_routes
            .checkpoint_selection()
            .map_err(checkpoint_admission_error)?;
        super::checkpoint_capture::runtime_selection::capture(
            self.runtime,
            &checkpoint_routes,
            selection,
            whole,
        )
    }

    pub(crate) fn finalize_captured_checkpoint(
        &self,
        captured: super::checkpoint_capture::CapturedCheckpointBasis,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DurableCheckpoint, crate::durability::data::RelationalRecoveryStop> {
        let checkpoint = captured.build_checkpoint_image(execution)?;
        self.runtime
            .publication_authority()
            .push_bounded_diagnostic(
                DiagnosticsScope::History,
                DiagnosticsArtifactKind::MinimalSummary,
                vec![checkpoint_created(&checkpoint)],
            );
        self.runtime.durability.push_checkpoint(checkpoint.clone());
        // Memory-only history has no external copy, including forks needed
        // by replay chains older than any retained checkpoint.
        if self.runtime.durability.medium().is_some() {
            self.runtime
                .history
                .compact_branch_transitions_through(checkpoint.coverage.up_to_branch_transition);
        }
        Ok(checkpoint)
    }

    #[cfg(test)]
    pub(crate) fn remove_durable_envelope_for_test(
        &self,
        commit_id: crate::history::data::CommitId,
    ) -> bool {
        self.runtime.durability.remove_log_commit(commit_id)
    }
}
