use worth_proof::TransitionOutcome;
use worth_signal::facade::branch::AdmittedSignalBranchBasis;
use worth_signal::facade::history::{RuntimeBranch, RuntimeBranchId, SignalBranchRetirementReason};

use crate::boundary::errors::WorthSignalJsError;

use super::worker_branch_command_model::{
    WorkerCloseoutEffectBranchReceipt, WorkerCloseoutEffectBranchRequest,
    WorkerRetireBranchReceipt, WorkerRetireBranchRequest, WorkerRetireBranchesReceipt,
    WorkerRetireBranchesRequest,
};
use super::worker_branch_commands::{expect_success, require_basis};
use super::worker_branch_snapshot_retirement::WorkerBranchSnapshotRetirement;
use super::RuntimeCore;

impl RuntimeCore {
    pub fn retire_worker_branches(
        &mut self,
        request: WorkerRetireBranchesRequest,
    ) -> Result<WorkerRetireBranchesReceipt, WorthSignalJsError> {
        let mut terminal_bases = Vec::with_capacity(request.retirements.len());
        let mut native_requests = Vec::with_capacity(request.retirements.len());
        for retirement in &request.retirements {
            let terminal_basis = self.worker_branch_basis(retirement.branch_id)?;
            require_basis(
                &retirement.expected_basis,
                &terminal_basis,
                "retireBranches",
            )?;
            terminal_bases.push(terminal_basis);
            let (branch, basis, reason) = self.native_retirement_basis(retirement)?;
            let snapshots = WorkerBranchSnapshotRetirement::admitted_for(
                &self.admitted_runtime_snapshots,
                retirement.branch_id,
            );
            native_requests.push((branch, basis, snapshots, reason));
        }
        let branch_ids = request
            .retirements
            .iter()
            .map(|retirement| retirement.branch_id)
            .collect::<Vec<_>>();
        let plan = match expect_success(
            self.runtime
                .plan_signal_branch_retirement_batch_releasing_snapshots(native_requests),
            "plan worker retirement batch",
        ) {
            Ok(plan) => plan,
            Err(error) => return Err(error),
        };
        WorkerBranchSnapshotRetirement::release(self, &branch_ids);
        let receipt = match expect_success(
            self.runtime.retire_signal_branch_batch(plan),
            "retire worker branch batch",
        ) {
            Ok(receipt) => receipt,
            Err(error) => return Err(error),
        };
        let retirements = request
            .retirements
            .into_iter()
            .zip(terminal_bases)
            .zip(receipt.receipts().iter().cloned())
            .map(|((retirement, terminal_basis), native_receipt)| {
                self.reclaim_worker_branch_companion_state(retirement.branch_id);
                worker_retirement_receipt(retirement, terminal_basis, native_receipt)
            })
            .collect();
        Ok(WorkerRetireBranchesReceipt { retirements })
    }

    pub fn closeout_worker_effect_branch(
        &mut self,
        request: WorkerCloseoutEffectBranchRequest,
    ) -> Result<WorkerCloseoutEffectBranchReceipt, WorthSignalJsError> {
        let effect_request = request.effect_retirement;
        let dependency_request = request.dependency_basis_retirement;
        let mut retirement_branch_ids = vec![effect_request.branch_id];
        if let Some(dependency) = &dependency_request {
            retirement_branch_ids.push(dependency.branch_id);
        }
        self.require_effect_closeout_canonical_target(
            request.canonical_transaction.branch_id,
            effect_request.branch_id,
            &retirement_branch_ids,
        )?;
        let terminal_basis = self.worker_branch_basis(effect_request.branch_id)?;
        require_basis(
            &effect_request.expected_basis,
            &terminal_basis,
            "closeoutEffectBranch.effectRetirement",
        )?;
        let dependency_terminal_basis = if let Some(dependency) = &dependency_request {
            let observed = self.worker_branch_basis(dependency.branch_id)?;
            require_basis(
                &dependency.expected_basis,
                &observed,
                "closeoutEffectBranch.dependencyBasisRetirement",
            )?;
            Some(observed)
        } else {
            None
        };
        let (effect_branch, effect_basis, effect_reason) =
            self.native_retirement_basis(&effect_request)?;
        let effect_snapshots = WorkerBranchSnapshotRetirement::admitted_for(
            &self.admitted_runtime_snapshots,
            effect_request.branch_id,
        );
        let mut retirement_requests =
            vec![(effect_branch, effect_basis, effect_snapshots, effect_reason)];
        if let Some(dependency) = &dependency_request {
            let (branch, basis, reason) = self.native_retirement_basis(dependency)?;
            let snapshots = WorkerBranchSnapshotRetirement::admitted_for(
                &self.admitted_runtime_snapshots,
                dependency.branch_id,
            );
            retirement_requests.push((branch, basis, snapshots, reason));
        }
        let retirement_plan = match expect_success(
            self.runtime
                .plan_signal_branch_retirement_batch_releasing_snapshots(retirement_requests),
            "plan worker effect closeout retirement batch",
        ) {
            Ok(plan) => plan,
            Err(error) => return Err(error),
        };

        let canonical_transaction =
            match self.apply_transaction_to_worker_branch(request.canonical_transaction) {
                Ok(receipt) => receipt,
                Err(error) => {
                    drop(retirement_plan);
                    return Err(error);
                }
            };
        WorkerBranchSnapshotRetirement::release(self, &retirement_branch_ids);
        let retired = match self.runtime.retire_signal_branch_batch(retirement_plan) {
            TransitionOutcome::Success(receipt) => receipt,
            other => {
                return Err(WorthSignalJsError::internal(format!(
                    "prevalidated effect retirement batch changed before execution: {other:?}"
                )));
            }
        };
        let receipts = retired.receipts();
        self.reclaim_worker_branch_companion_state(effect_request.branch_id);
        let effect_retirement =
            worker_retirement_receipt(effect_request, terminal_basis, receipts[0].clone());
        let dependency_basis_retirement = dependency_request.map(|dependency| {
            self.reclaim_worker_branch_companion_state(dependency.branch_id);
            worker_retirement_receipt(
                dependency,
                dependency_terminal_basis.expect("dependency basis was prevalidated"),
                receipts[1].clone(),
            )
        });
        Ok(WorkerCloseoutEffectBranchReceipt {
            canonical_transaction,
            effect_retirement,
            dependency_basis_retirement,
        })
    }

    /// The canonical target of an effect closeout is the branch the effect's
    /// work settles into: the nearest ancestor of the effect branch that this
    /// closeout does not retire. Resource effects fork from whatever branch
    /// the product treats as canonical (the root, or a branch head the app
    /// switched to), so the target is defined by lineage, not by root-ness.
    /// A retiring branch cannot receive the canonical transaction (its truth
    /// is released in the same closeout), and an unrelated live branch (a
    /// coordinator, a sibling effect) is not where this effect's work belongs.
    fn require_effect_closeout_canonical_target(
        &self,
        canonical_target_id: u64,
        effect_branch_id: u64,
        retiring_branch_ids: &[u64],
    ) -> Result<(), WorthSignalJsError> {
        if self
            .runtime
            .branch_handle(RuntimeBranchId(canonical_target_id))
            .is_none()
        {
            return Err(WorthSignalJsError::invalid_input(format!(
                "closeoutEffectBranch references unknown canonical transaction branch `{canonical_target_id}`"
            )));
        }
        if retiring_branch_ids.contains(&canonical_target_id) {
            return Err(WorthSignalJsError::invalid_input(format!(
                "closeoutEffectBranch canonical transaction target `{canonical_target_id}` is retired by this closeout; the canonical target must be the nearest surviving ancestor of effect branch `{effect_branch_id}`"
            )));
        }
        let mut cursor = self
            .runtime
            .branch_handle(RuntimeBranchId(effect_branch_id))
            .ok_or_else(|| {
                WorthSignalJsError::invalid_input(format!(
                    "closeoutEffectBranch references unknown effect branch `{effect_branch_id}`"
                ))
            })?;
        loop {
            let Some(parent_id) = cursor.parent_branch_id else {
                return Err(WorthSignalJsError::invalid_input(format!(
                    "closeoutEffectBranch effect branch `{effect_branch_id}` has no surviving ancestor to settle into"
                )));
            };
            if !retiring_branch_ids.contains(&parent_id.0) {
                if parent_id.0 == canonical_target_id {
                    return Ok(());
                }
                return Err(WorthSignalJsError::invalid_input(format!(
                    "closeoutEffectBranch canonical transaction target `{canonical_target_id}` must be the nearest surviving ancestor `{}` of effect branch `{effect_branch_id}`",
                    parent_id.0
                )));
            }
            cursor = self.runtime.branch_handle(parent_id).ok_or_else(|| {
                WorthSignalJsError::internal(format!(
                    "closeoutEffectBranch retiring branch `{}` is in the lineage of effect branch `{effect_branch_id}` but has no branch handle",
                    parent_id.0
                ))
            })?;
        }
    }

    fn native_retirement_basis(
        &self,
        request: &WorkerRetireBranchRequest,
    ) -> Result<
        (
            RuntimeBranch,
            AdmittedSignalBranchBasis,
            SignalBranchRetirementReason,
        ),
        WorthSignalJsError,
    > {
        let branch = self
            .runtime
            .branch_handle(RuntimeBranchId(request.branch_id))
            .ok_or_else(|| {
                WorthSignalJsError::invalid_input(
                    "closeoutEffectBranch references an unknown retirement branch",
                )
            })?;
        let native_basis = self.native_branch_basis_by_id(request.branch_id)?;
        Ok((branch, native_basis, request.reason.into()))
    }
}

fn worker_retirement_receipt(
    request: WorkerRetireBranchRequest,
    terminal_basis: super::WorkerBranchBasisReceipt,
    receipt: worth_signal::facade::SignalBranchRetirementReceipt,
) -> WorkerRetireBranchReceipt {
    WorkerRetireBranchReceipt {
        retired_branch_id: request.branch_id,
        parent_branch_id: receipt.parent_branch_id().0,
        terminal_basis,
        closeout_digest: receipt.closeout_digest().to_owned(),
        reclaimed_branch_state_count: receipt.reclaimed_branch_state_count(),
        reclaimed_snapshot_state_count: receipt.reclaimed_snapshot_state_count(),
        reclaimed_runtime_meta_count: receipt.reclaimed_runtime_meta_count(),
        retained_proof_record_count: receipt.retained_proof_record_count(),
    }
}
