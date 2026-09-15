use std::sync::{Arc, Mutex};

use crate::boundary::errors::WorthSignalJsError;
use crate::recipe::model::TransactionOp;
use crate::runtime::summaries::RunSummary;
use worth_proof::TransitionOutcome;
use worth_signal::facade::branch::AdmittedSignalBranchBasis;
use worth_signal::facade::history::{RuntimeBranch, RuntimeBranchId, SignalBranchRetirementReason};
use worth_signal::facade::SignalError;

use super::certification_digest::canonical_certification_digest;
use super::state::{BranchRuntimeState, PendingCallbackDependencyPatch};
use super::transactions::apply::{
    apply_pending_dependency_patches_in_transaction, apply_set_changes,
};
use super::worker_branch_command_model::{
    WorkerApplyTransactionToBranchReceipt, WorkerApplyTransactionToBranchRequest,
    WorkerBranchBasisReceipt, WorkerBranchRetirementReason, WorkerForkBranchReceipt,
    WorkerForkBranchRequest,
};
use super::RuntimeCore;

impl RuntimeCore {
    pub fn worker_branch_basis(
        &self,
        branch_id: u64,
    ) -> Result<WorkerBranchBasisReceipt, WorthSignalJsError> {
        let branch = self
            .runtime
            .branch_handle(RuntimeBranchId(branch_id))
            .ok_or_else(|| unknown_branch(branch_id))?;
        let native_basis = self.native_branch_basis(branch.clone())?;
        let state = self.state_for_branch(branch_id);
        let authored_state_digest = canonical_certification_digest(&(
            &state.store.sources,
            state.authored_graph_generation,
        ))?;
        Ok(worker_basis(
            branch,
            native_basis,
            state,
            authored_state_digest,
        )?)
    }

    pub fn fork_worker_branch(
        &mut self,
        request: WorkerForkBranchRequest,
    ) -> Result<WorkerForkBranchReceipt, WorthSignalJsError> {
        let parent_basis = self.worker_branch_basis(request.parent_branch_id)?;
        require_basis(&request.expected_parent_basis, &parent_basis, "forkBranch")?;
        let native_parent_basis = self.native_branch_basis_by_id(request.parent_branch_id)?;
        let parent_state = self.state_for_branch(request.parent_branch_id);
        let fork = self
            .runtime
            .fork_signal_branch(request.name, &native_parent_basis)
            .map_err(|error| {
                WorthSignalJsError::invalid_input(format!("fork worker branch denied: {error:?}"))
            })?;
        let branch = fork.created_branch().clone();
        self.branch_states.insert(branch.id.0, parent_state);
        let created_basis = self.worker_branch_basis(branch.id.0)?;
        Ok(WorkerForkBranchReceipt {
            branch,
            parent_basis,
            created_basis,
        })
    }

    pub fn apply_transaction_to_worker_branch(
        &mut self,
        request: WorkerApplyTransactionToBranchRequest,
    ) -> Result<WorkerApplyTransactionToBranchReceipt, WorthSignalJsError> {
        let active_branch_id_before = self.runtime.current_branch().id.0;
        if request.branch_id == active_branch_id_before {
            return Err(WorthSignalJsError::invalid_input(format!(
                "applyTransactionToBranch denies active branch target `{}`",
                request.branch_id
            )));
        }
        let before_basis = self.worker_branch_basis(request.branch_id)?;
        require_basis(
            &request.expected_basis,
            &before_basis,
            "applyTransactionToBranch",
        )?;
        let branch = self
            .runtime
            .branch_handle(RuntimeBranchId(request.branch_id))
            .ok_or_else(|| unknown_branch(request.branch_id))?;
        let native_basis = self.native_branch_basis(branch.clone())?;

        let active_state = self.snapshot_branch_state();
        let target_state = self
            .branch_states
            .get(&request.branch_id)
            .cloned()
            .ok_or_else(|| unknown_branch(request.branch_id))?;
        let target_result = (|| {
            self.install_companion_state(&target_state)?;
            self.validate_targeted_transaction_shape(&request.transaction_ops)?;
            let changes = self.collect_changes(&request.transaction_ops)?;
            let store = self.store.clone();
            let dense_grids = self.dense_grids.clone();
            let evaluator = self.evaluator();
            let standing_demand = self.standing_demand_nodes();
            let dependency_patches = Arc::new(Mutex::new(
                None::<(Vec<PendingCallbackDependencyPatch>, u64)>,
            ));
            let dependency_patches_for_tx = dependency_patches.clone();
            let outcome =
                self.runtime
                    .advance_signal_branch(&mut self.store, &native_basis, move |tx| {
                        apply_set_changes(tx, &store, &dense_grids, &changes)?;
                        tx.evaluate_dirty(&evaluator)?;
                        tx.evaluate_demand(&evaluator, &standing_demand)?;
                        let patches = apply_pending_dependency_patches_in_transaction(tx, &store)?;
                        *dependency_patches_for_tx.lock().map_err(|_| {
                            SignalError::internal("dependency patch receipt mutex poisoned")
                        })? = Some(patches);
                        Ok(())
                    });
            let transaction = outcome.map_err(|error| {
                WorthSignalJsError::invalid_input(format!(
                    "execute targeted worker transaction denied: {error:?}"
                ))
            })?;
            let (pending, runtime_read_breadth) = dependency_patches
                .lock()
                .map_err(|_| {
                    WorthSignalJsError::internal("dependency patch receipt mutex poisoned")
                })?
                .take()
                .unwrap_or_default();
            self.record_committed_callback_dependency_patches(pending, runtime_read_breadth)?;
            let committed_target_state = BranchRuntimeState {
                metadata: self.snapshot_branch_metadata(),
                store: self.lock_store()?.snapshot(&self.catalog),
                authored_graph_generation: target_state.authored_graph_generation.saturating_add(1),
            };
            Ok::<_, WorthSignalJsError>((transaction.into_parts().1, committed_target_state))
        })();
        let restoration = self.install_companion_state(&active_state);
        let (transaction, committed_target_state) = match target_result {
            Ok(result) => {
                restoration?;
                result
            }
            Err(error) => {
                restoration?;
                return Err(error);
            }
        };
        self.branch_states
            .insert(request.branch_id, committed_target_state);
        let active_branch_id_after = self.runtime.current_branch().id.0;
        let after_basis = self.worker_branch_basis(request.branch_id)?;
        Ok(WorkerApplyTransactionToBranchReceipt {
            before_basis,
            after_basis,
            active_branch_id_before,
            active_branch_id_after,
            run_summary: run_summary(&transaction),
        })
    }

    fn install_companion_state(
        &mut self,
        state: &BranchRuntimeState,
    ) -> Result<(), WorthSignalJsError> {
        self.ensure_callback_snapshot_availability(&state.store)?;
        self.restore_branch_metadata(state.metadata.clone());
        self.lock_store()?.restore_snapshot(state.store.clone());
        self.sync_callback_diagnostics_from_store()
    }

    pub(super) fn native_branch_basis_by_id(
        &self,
        branch_id: u64,
    ) -> Result<AdmittedSignalBranchBasis, WorthSignalJsError> {
        let branch = self
            .runtime
            .branch_handle(RuntimeBranchId(branch_id))
            .ok_or_else(|| unknown_branch(branch_id))?;
        self.native_branch_basis(branch)
    }

    pub(super) fn native_branch_basis(
        &self,
        branch: RuntimeBranch,
    ) -> Result<AdmittedSignalBranchBasis, WorthSignalJsError> {
        self.runtime
            .observe_signal_branch_basis(branch)
            .map_err(|error| {
                WorthSignalJsError::invalid_input(format!(
                    "read worker branch basis denied: {error:?}"
                ))
            })
    }

    fn validate_targeted_transaction_shape(
        &self,
        ops: &[TransactionOp],
    ) -> Result<(), WorthSignalJsError> {
        for op in ops {
            match op {
                TransactionOp::Set { id, .. } | TransactionOp::SetWithRegions { id, .. } => {
                    if !self.catalog.contains_key(id) {
                        return Err(unknown_target_signal(id));
                    }
                }
                TransactionOp::SetMany { values } => {
                    for value in values {
                        if !self.catalog.contains_key(&value.id) {
                            return Err(unknown_target_signal(&value.id));
                        }
                    }
                }
                TransactionOp::SetManyWithRegions { values } => {
                    for value in values {
                        if !self.catalog.contains_key(&value.id) {
                            return Err(unknown_target_signal(&value.id));
                        }
                    }
                }
                TransactionOp::SetManyKeyed { family_id, .. } => {
                    return Err(WorthSignalJsError::invalid_input(format!(
                        "branch-targeted transaction cannot materialize keyed family `{family_id}`; publish authored keys before forking"
                    )));
                }
                TransactionOp::SetPackedGridRgba {
                    family_id,
                    width,
                    height,
                    ..
                } => {
                    let Some(family) = self.dense_grids.get(family_id) else {
                        return Err(WorthSignalJsError::invalid_input(format!(
                            "branch-targeted transaction references unknown dense family `{family_id}`"
                        )));
                    };
                    if family.width != *width || family.height != *height {
                        return Err(WorthSignalJsError::invalid_input(format!(
                            "branch-targeted dense family `{family_id}` shape does not match its authored graph"
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

impl From<WorkerBranchRetirementReason> for SignalBranchRetirementReason {
    fn from(reason: WorkerBranchRetirementReason) -> Self {
        match reason {
            WorkerBranchRetirementReason::Rejected => Self::Rejected,
            WorkerBranchRetirementReason::Merged => Self::Merged,
            WorkerBranchRetirementReason::Superseded => Self::Superseded,
            WorkerBranchRetirementReason::DependencyCancellation => Self::DependencyCancellation,
            WorkerBranchRetirementReason::ProjectionRebuild => Self::ProjectionRebuild,
        }
    }
}

fn worker_basis(
    branch: RuntimeBranch,
    native_basis: AdmittedSignalBranchBasis,
    state: BranchRuntimeState,
    authored_state_digest: String,
) -> Result<WorkerBranchBasisReceipt, WorthSignalJsError> {
    let target = native_basis
        .observation()
        .target()
        .as_basis()
        .ok_or_else(|| {
            WorthSignalJsError::internal("live Signal basis unexpectedly has an empty target")
        })?;
    let native_head_digest =
        canonical_certification_digest(&native_basis.observation().canonical_encoding())?;
    Ok(WorkerBranchBasisReceipt {
        branch_id: branch.id.0,
        branch_name: branch.name,
        snapshot_id: target.snapshot_id(),
        native_head_generation: native_basis.observation().generation().get(),
        native_head_digest,
        authored_graph_generation: state.authored_graph_generation,
        authored_state_digest,
    })
}

pub(super) fn require_basis(
    expected: &WorkerBranchBasisReceipt,
    observed: &WorkerBranchBasisReceipt,
    operation: &str,
) -> Result<(), WorthSignalJsError> {
    if expected == observed {
        return Ok(());
    }
    Err(WorthSignalJsError::invalid_input(format!(
        "{operation} denied a stale worker branch basis: expected generation {}/{}, observed {}/{}; expected {expected:?}; observed {observed:?}",
        expected.native_head_generation,
        expected.authored_graph_generation,
        observed.native_head_generation,
        observed.authored_graph_generation,
    )))
}

pub(super) fn expect_success<T: std::fmt::Debug, D: std::fmt::Debug>(
    outcome: TransitionOutcome<T, D>,
    operation: &str,
) -> Result<T, WorthSignalJsError> {
    match outcome {
        TransitionOutcome::Success(value) => Ok(value),
        other => Err(outcome_error(operation, other)),
    }
}

fn outcome_error<
    T: std::fmt::Debug,
    D: std::fmt::Debug,
    De: std::fmt::Debug,
    S: std::fmt::Debug,
    R: std::fmt::Debug,
    F: std::fmt::Debug,
>(
    operation: &str,
    outcome: TransitionOutcome<T, D, De, S, R, F>,
) -> WorthSignalJsError {
    WorthSignalJsError::invalid_input(format!("{operation} denied: {outcome:?}"))
}

pub(super) fn unknown_branch(branch_id: u64) -> WorthSignalJsError {
    WorthSignalJsError::invalid_input(format!("unknown worker branch `{branch_id}`"))
}

fn unknown_target_signal(id: &str) -> WorthSignalJsError {
    WorthSignalJsError::invalid_input(format!(
        "branch-targeted transaction references unknown authored signal `{id}`"
    ))
}

fn run_summary(result: &worth_signal::facade::TransactionResult) -> RunSummary {
    RunSummary {
        touched_nodes: result.touched_nodes,
        nodes_evaluated: result.evaluation_summary.nodes_evaluated,
        nodes_recomputed: result.evaluation_summary.nodes_recomputed,
        nodes_suppressed: result.evaluation_summary.nodes_suppressed,
        plans_built: result.evaluation_summary.plans_built,
        stages_executed: result.evaluation_summary.stages_executed,
        total_nanos: result.timing.total_nanos.to_string(),
        evaluation_nanos: result.timing.evaluation_nanos.to_string(),
        commit_nanos: result.timing.commit_nanos.to_string(),
    }
}
