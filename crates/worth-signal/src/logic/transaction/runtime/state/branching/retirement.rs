use std::collections::BTreeSet;
use worth_proof::TransitionOutcome;

use crate::branch::{
    AdmittedSignalBranchBasis, PlannedSignalBranchRetirement, SignalBranchRetirementDenial,
    SignalBranchRetirementReason, SignalBranchRetirementReceipt,
};
use crate::logic::transaction::canonical_digest;
use crate::state::{SignalBranchHandle, SignalBranchId};

use super::super::runtime_state::SignalRuntime;
use super::transaction_head::SignalBranchTransactionHead;

#[derive(Debug, Clone)]
pub struct SignalBranchRetirementRequest {
    branch: SignalBranchHandle,
    expected_head: SignalBranchTransactionHead,
    reason: SignalBranchRetirementReason,
}

impl SignalBranchRetirementRequest {
    pub(crate) fn new(
        branch: SignalBranchHandle,
        expected_head: SignalBranchTransactionHead,
        reason: SignalBranchRetirementReason,
    ) -> Self {
        Self {
            branch,
            expected_head,
            reason,
        }
    }

    pub(crate) fn branch(&self) -> &SignalBranchHandle {
        &self.branch
    }

    pub(crate) fn expected_head(&self) -> &SignalBranchTransactionHead {
        &self.expected_head
    }

    pub(crate) fn reason(&self) -> SignalBranchRetirementReason {
        self.reason
    }
}

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn plan_branch_retirement_with_canonical_basis_after(
        &mut self,
        request: SignalBranchRetirementRequest,
        basis: AdmittedSignalBranchBasis,
        retired_before: &BTreeSet<SignalBranchId>,
        admitted_allowance: u32,
    ) -> TransitionOutcome<PlannedSignalBranchRetirement, SignalBranchRetirementDenial> {
        self.with_telemetry(|telemetry| telemetry.transaction.branch_retirement_plan_count += 1);
        if let Err(denial) = self.validate_retirement_request_after_with_admitted(
            &request,
            retired_before,
            admitted_allowance,
        ) {
            return TransitionOutcome::denied(denial);
        }
        let active_leases = self
            .branches
            .branch_admitted_retention_count(request.branch.id);
        let shared_holders = basis.shared_holder_count();
        if active_leases != admitted_allowance {
            return TransitionOutcome::denied(
                SignalBranchRetirementDenial::RetainedAdmittedBasis {
                    branch_id: request.branch.id,
                    active_leases,
                },
            );
        }
        if shared_holders != 1 {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::SharedAdmittedBasis {
                branch_id: request.branch.id,
                shared_holders,
            });
        }
        let child_count = self.branches.branch_children(request.branch.id).len() as u32;
        let terminal_basis_digest = canonical_digest(&basis.observation().canonical_encoding());
        TransitionOutcome::success(PlannedSignalBranchRetirement {
            branch: request.branch,
            reason: request.reason,
            terminal_basis_digest,
            planned_child_membership_count: child_count,
            admitted_basis: basis,
        })
    }

    pub(crate) fn retire_branch(
        &mut self,
        plan: PlannedSignalBranchRetirement,
    ) -> TransitionOutcome<SignalBranchRetirementReceipt, SignalBranchRetirementDenial> {
        let branch = plan.branch.clone();
        let expected_generation = plan.admitted_basis.observation().generation().get();
        let live = match self.signal_branch_observation(&branch) {
            Ok(live) => live,
            Err(_) => {
                return TransitionOutcome::denied(SignalBranchRetirementDenial::UnknownBranch {
                    branch_id: branch.id,
                })
            }
        };
        if live.compare(plan.admitted_basis.observation()).is_err() {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::StaleBranchHead {
                expected_generation,
                observed_generation: live.generation().get(),
            });
        }
        let current_head = match self.branch_transaction_head(branch.clone()) {
            TransitionOutcome::Success(head) => head,
            _ => {
                return TransitionOutcome::denied(SignalBranchRetirementDenial::UnknownBranch {
                    branch_id: branch.id,
                })
            }
        };
        let request = SignalBranchRetirementRequest::new(branch.clone(), current_head, plan.reason);
        if let Err(denial) =
            self.validate_retirement_request_after_with_admitted(&request, &BTreeSet::new(), 1)
        {
            self.with_telemetry(|telemetry| {
                telemetry.transaction.branch_retirement_denial_count += 1
            });
            return TransitionOutcome::denied(denial);
        }
        let shared_holders = plan.admitted_basis.shared_holder_count();
        if shared_holders != 1 {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::SharedAdmittedBasis {
                branch_id: branch.id,
                shared_holders,
            });
        }
        let reason = plan.reason;
        let terminal_basis_digest = plan.terminal_basis_digest;
        drop(plan.admitted_basis);
        let ancestry = self
            .branches
            .branch_ancestry_state(branch.id)
            .expect("validated retirement must retain ancestry")
            .clone();
        let parent_branch_id = ancestry
            .parent_branch_id()
            .expect("canonical branch retirement is denied during planning");
        let terminal_head_snapshot_id = self.branch_head_snapshot_id(branch.id);
        let reclaimed = self
            .branches
            .retire_stored_branch(branch.id)
            .expect("validated retirement must reclaim a stored branch");
        let closeout_digest = canonical_digest(&(
            branch.id,
            parent_branch_id,
            ancestry.forked_from_snapshot_id(),
            terminal_head_snapshot_id,
            reason,
            terminal_basis_digest.as_str(),
        ));
        let receipt = SignalBranchRetirementReceipt {
            retired_branch: branch.clone(),
            parent_branch_id,
            forked_from_snapshot_id: ancestry.forked_from_snapshot_id(),
            terminal_head_snapshot_id,
            reason,
            terminal_basis_digest,
            closeout_digest,
            reclaimed_branch_state_count: reclaimed.branch_state_count,
            reclaimed_snapshot_state_count: reclaimed.snapshot_state_count,
            reclaimed_runtime_meta_count: reclaimed.runtime_meta_count,
            retained_proof_record_count: 1,
        };
        self.branches.retain_retirement_receipt(receipt.clone());
        self.project_branch_catalog();
        self.with_telemetry(|telemetry| {
            telemetry.transaction.branch_retirement_execution_count += 1;
            telemetry
                .transaction
                .branch_retirement_reclaimed_branch_state_count +=
                u64::from(receipt.reclaimed_branch_state_count);
            telemetry
                .transaction
                .branch_retirement_reclaimed_snapshot_state_count +=
                u64::from(receipt.reclaimed_snapshot_state_count);
            telemetry
                .transaction
                .branch_retirement_reclaimed_runtime_meta_count +=
                u64::from(receipt.reclaimed_runtime_meta_count);
            telemetry.transaction.branch_retirement_retained_proof_count += 1;
        });
        crate::diagnostics::recorder::record_snapshot_event(
            &mut self.graph,
            crate::diagnostics::replay::ReplayEventKind::BranchRetired,
            None,
            format!(
                "retired branch `{}` with closeout `{}`",
                branch.name,
                receipt.closeout_digest()
            ),
        );
        TransitionOutcome::success(receipt)
    }
}
