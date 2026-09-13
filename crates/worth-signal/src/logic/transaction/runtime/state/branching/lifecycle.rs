use crate::data::error::SignalError;
use crate::state::{SignalBranchHandle, SignalBranchId, SignalSnapshotId};

use super::super::runtime_state::{
    AuthorityTransferPacket, BranchLifecycleTransfer, SignalRuntime,
};
use super::{SignalBranchForkDenial, SignalBranchForkRequest};

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn create_branch(
        &mut self,
        name: impl Into<String>,
    ) -> Result<SignalBranchHandle, SignalError> {
        match self.fork_branch(SignalBranchForkRequest::from_current_branch_head(name)) {
            worth_proof::TransitionOutcome::Success(receipt) => {
                Ok(receipt.created_branch().clone())
            }
            worth_proof::TransitionOutcome::Denied(denial) => {
                Err(Self::fork_denial_to_signal_error(denial))
            }
            other => Err(SignalError::internal(format!(
                "unexpected non-terminal fork outcome for compatibility create_branch: {other:?}"
            ))),
        }
    }

    pub fn switch_branch(&mut self, branch: SignalBranchHandle) -> Result<(), SignalError> {
        if self.owner_services.is_sealed() {
            return Err(SignalError::invalid_input(
                "branch selection cannot change after owner-service sealing",
            ));
        }
        let current = self.graph.current_branch();
        let preserved_transaction = self.telemetry_snapshot().transaction;
        if branch.id == current.id {
            return Ok(());
        }
        let target_state = self
            .branches
            .branch_state(branch.id)
            .ok_or_else(|| SignalError::unknown_branch(Some(branch.id), branch.name.clone()))?;
        self.ensure_branch_state_managed_queue_transfer_allowed(target_state)?;
        let Some(packet) = self.branches.take_stored_branch_transfer(branch.id) else {
            return Err(SignalError::unknown_branch(Some(branch.id), branch.name));
        };
        self.graph.interrupt_observation_at_boundary();
        let mut state = packet.into_state();
        state.graph_mut().interrupt_observation_at_boundary();
        let current_state = self.take_heavy_active_branch_state()?;
        self.branches.store_branch_state(current_state);
        self.apply_branch_lifecycle_transfer(BranchLifecycleTransfer::Move(
            AuthorityTransferPacket::new(branch.id, state),
        ))?;
        self.with_telemetry(|telemetry| {
            Self::merge_global_transaction_telemetry(
                preserved_transaction,
                &mut telemetry.transaction,
            );
            telemetry.transaction.move_transfer_count += 2;
        });
        self.project_branch_catalog();
        crate::diagnostics::recorder::record_snapshot_event(
            &mut self.graph,
            crate::diagnostics::replay::ReplayEventKind::BranchSwitched,
            None,
            format!("switched from `{}` to `{}`", current.name, branch.name),
        );
        crate::diagnostics::recorder::record_branch_switch_lineage(
            &mut self.graph,
            current.id,
            branch.id,
            current.name.to_string(),
            branch.name.clone(),
        );
        Ok(())
    }

    pub fn current_branch(&self) -> SignalBranchHandle {
        if self.owner_services.is_sealed() {
            return self
                .owner_services
                .selected_branch_handle()
                .expect("a sealed owner retains its selected branch identity");
        }
        self.graph.current_branch()
    }

    pub fn known_branches(&self) -> Vec<SignalBranchHandle> {
        if self.owner_services.is_sealed() {
            return self.owner_services.known_branch_handles();
        }
        self.branches.known_branches()
    }

    pub fn branch_handle(&self, branch_id: SignalBranchId) -> Option<SignalBranchHandle> {
        if self.owner_services.is_sealed() {
            return self.owner_services.branch_handle(branch_id);
        }
        self.branches.branch_handle(branch_id)
    }

    pub fn branch_ancestry(&self, branch_id: SignalBranchId) -> Vec<SignalBranchHandle> {
        if self.owner_services.is_sealed() {
            return self.owner_services.branch_ancestry(branch_id);
        }
        self.branches.branch_ancestry(branch_id)
    }

    pub fn branch_head_snapshot_id(&self, branch_id: SignalBranchId) -> Option<SignalSnapshotId> {
        if self.owner_services.is_sealed() {
            return self.owner_services.branch_head_snapshot_id(branch_id);
        }
        self.branches.branch_head_snapshot_id(branch_id)
    }

    fn replay_graph_for_branch(
        &self,
        branch_id: SignalBranchId,
    ) -> Option<&crate::data::graph::SignalGraph> {
        self.branches
            .replay_graph(branch_id, self.graph.current_branch().id, &self.graph)
    }

    pub fn replay_for_branch(&self, branch_id: SignalBranchId) -> crate::diagnostics::ReplayView {
        self.replay_graph_for_branch(branch_id)
            .map(|graph| graph.replay_for_branch(branch_id))
            .unwrap_or_default()
    }

    #[cfg(test)]
    pub(crate) fn clear_branch_merge_boundary_for_test(
        &mut self,
        branch_id: SignalBranchId,
    ) -> Result<(), SignalError> {
        if self.graph.current_branch().id == branch_id {
            let mut state = self.capture_heavy_branch_state()?;
            state.clear_merge_boundary_proof();
            self.branches.observe_active_branch_state(&state);
            return Ok(());
        }
        let Some(()) = self
            .branches
            .with_stored_branch_state_mut(branch_id, |state| {
                state.clear_merge_boundary_proof();
            })
        else {
            return Err(SignalError::unknown_branch(Some(branch_id), "test-branch"));
        };
        Ok(())
    }
}

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn fork_denial_to_signal_error(denial: SignalBranchForkDenial) -> SignalError {
        match denial {
            SignalBranchForkDenial::InvalidBranchIdentity => {
                SignalError::invalid_input("invalid Signal branch identity")
            }
            SignalBranchForkDenial::BranchIdentityExhausted => {
                SignalError::internal("Signal branch identity exhausted")
            }
            SignalBranchForkDenial::UnknownParentBranch { parent_branch_id } => {
                SignalError::unknown_branch(Some(parent_branch_id), "fork-parent")
            }
            SignalBranchForkDenial::UnknownForkSnapshot {
                parent_branch_id,
                snapshot_id,
            } => SignalError::invalid_input(format!(
                "fork snapshot `{}:{}` is not tracked by the runtime",
                parent_branch_id.0, snapshot_id.0
            )),
            SignalBranchForkDenial::SnapshotBasisMismatch {
                requested_snapshot_id,
                provided_snapshot_id,
            } => SignalError::invalid_input(format!(
                "fork request expected snapshot `{}` but received snapshot `{}`",
                requested_snapshot_id.0, provided_snapshot_id.0
            )),
            SignalBranchForkDenial::SnapshotPayloadRequiredForFork { request } => {
                SignalError::invalid_input(format!(
                    "fork request `{}` declares snapshot basis but no snapshot payload was provided",
                    request.branch_name()
                ))
            }
            SignalBranchForkDenial::IncompatibleForkSnapshotLineage {
                parent_branch_id,
                snapshot_branch_id,
                snapshot_id,
            } => SignalError::invalid_input(format!(
                "snapshot `{}` belongs to branch `{}` and cannot seed a fork from branch `{}`",
                snapshot_id.0, snapshot_branch_id.0, parent_branch_id.0
            )),
            SignalBranchForkDenial::ManagedQueueBranchTransferDenied {
                bound_queue_count,
            } => SignalError::managed_queue_branch_transfer_denied(bound_queue_count),
        }
    }
}
