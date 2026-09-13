use std::collections::BTreeSet;

use worth_proof::TransitionOutcome;

use crate::branch::AdmittedSignalBranchBasis;
use crate::branch::AdmittedSignalBranchSnapshot;
use crate::state::SignalBranchHandle;

use super::super::runtime_state::SignalRuntime;
use super::{
    BranchTargetedTransactionDenial, PlannedSignalBranchRetirement, SignalBranchRetirementDenial,
    SignalBranchRetirementReason, SignalBranchRetirementReceipt, SignalBranchRetirementRequest,
};

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Consume the sole admitted basis into a linear retirement plan.
    pub fn plan_signal_branch_retirement(
        &mut self,
        branch: SignalBranchHandle,
        expected: AdmittedSignalBranchBasis,
        reason: SignalBranchRetirementReason,
    ) -> TransitionOutcome<PlannedSignalBranchRetirement, SignalBranchRetirementDenial> {
        if self.owner_services.is_sealed() {
            return self
                .owner_services
                .plan_legacy_retirement(branch, expected, reason);
        }
        let branch_id = branch.id;
        let Some(branch) = self.branches.branch_handle(branch_id) else {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::UnknownBranch {
                branch_id,
            });
        };
        let Ok(live) = self.signal_branch_observation(&branch) else {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::UnknownBranch {
                branch_id,
            });
        };
        if live.compare(expected.observation()).is_err() {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::CanonicalBasisMismatch);
        }
        let head = match self.branch_transaction_head(branch.clone()) {
            TransitionOutcome::Success(head) => head,
            TransitionOutcome::Denied(denial) => {
                return TransitionOutcome::denied(match denial {
                    BranchTargetedTransactionDenial::UnknownTargetBranch { branch_id } => {
                        SignalBranchRetirementDenial::UnknownBranch { branch_id }
                    }
                    BranchTargetedTransactionDenial::StaleTargetHead { expected, observed } => {
                        SignalBranchRetirementDenial::StaleBranchHead {
                            expected_generation: expected.generation(),
                            observed_generation: observed.generation(),
                        }
                    }
                    _ => SignalBranchRetirementDenial::UnknownBranch {
                        branch_id: branch.id,
                    },
                })
            }
            TransitionOutcome::Failed(_) => {
                return TransitionOutcome::denied(SignalBranchRetirementDenial::UnknownBranch {
                    branch_id: branch.id,
                })
            }
        };
        self.plan_branch_retirement_with_canonical_basis_after(
            SignalBranchRetirementRequest::new(branch, head, reason),
            expected,
            &BTreeSet::new(),
            1,
        )
    }

    /// Plan retirement while accounting for admitted snapshots that the
    /// caller will release before executing the returned linear plan.
    ///
    /// Borrowing the concrete snapshot authorities makes the allowance
    /// owner-verifiable. Planning has no effect on them; a denied plan leaves
    /// every supplied snapshot usable.
    pub fn plan_signal_branch_retirement_releasing_snapshots(
        &mut self,
        branch: SignalBranchHandle,
        expected: AdmittedSignalBranchBasis,
        releasing_snapshots: &[&AdmittedSignalBranchSnapshot],
        reason: SignalBranchRetirementReason,
    ) -> TransitionOutcome<PlannedSignalBranchRetirement, SignalBranchRetirementDenial> {
        if self.owner_services.is_sealed() {
            return self
                .owner_services
                .plan_legacy_retirement_releasing_snapshots(
                    branch,
                    expected,
                    releasing_snapshots,
                    reason,
                );
        }
        let branch_id = branch.id;
        let allowance = match self.retirement_snapshot_allowance(branch_id, releasing_snapshots) {
            Ok(allowance) => allowance,
            Err(denial) => return TransitionOutcome::denied(denial),
        };
        let Some(branch) = self.branches.branch_handle(branch_id) else {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::UnknownBranch {
                branch_id,
            });
        };
        let Ok(live) = self.signal_branch_observation(&branch) else {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::UnknownBranch {
                branch_id,
            });
        };
        if live.compare(expected.observation()).is_err() {
            return TransitionOutcome::denied(SignalBranchRetirementDenial::CanonicalBasisMismatch);
        }
        let head = match self.branch_transaction_head(branch.clone()) {
            TransitionOutcome::Success(head) => head,
            TransitionOutcome::Denied(BranchTargetedTransactionDenial::StaleTargetHead {
                expected,
                observed,
            }) => {
                return TransitionOutcome::denied(SignalBranchRetirementDenial::StaleBranchHead {
                    expected_generation: expected.generation(),
                    observed_generation: observed.generation(),
                })
            }
            TransitionOutcome::Denied(_) | TransitionOutcome::Failed(_) => {
                return TransitionOutcome::denied(SignalBranchRetirementDenial::UnknownBranch {
                    branch_id,
                })
            }
        };
        self.plan_branch_retirement_with_canonical_basis_after(
            SignalBranchRetirementRequest::new(branch, head, reason),
            expected,
            &BTreeSet::new(),
            allowance,
        )
    }

    pub fn retire_signal_branch(
        &mut self,
        plan: PlannedSignalBranchRetirement,
    ) -> TransitionOutcome<SignalBranchRetirementReceipt, SignalBranchRetirementDenial> {
        if let Some((_, _, lifecycle)) = self.sealed_owner_port_slots() {
            let cancellation = crate::branch::SignalOwnerCancellationSource::new();
            return lifecycle.retire_exact(plan, &cancellation.token());
        }
        self.retire_branch(plan)
    }

    pub(super) fn retirement_snapshot_allowance(
        &self,
        branch_id: crate::state::SignalBranchId,
        releasing_snapshots: &[&AdmittedSignalBranchSnapshot],
    ) -> Result<u32, SignalBranchRetirementDenial> {
        let expected_runtime_instance_id = self.branches.owner_runtime_instance_id();
        let mut unique_retentions = BTreeSet::new();
        for admitted_snapshot in releasing_snapshots {
            let observed_runtime_instance_id = admitted_snapshot.owner_runtime_instance_id();
            if observed_runtime_instance_id != expected_runtime_instance_id {
                return Err(SignalBranchRetirementDenial::ForeignRetirementSnapshot {
                    expected_runtime_instance_id,
                    observed_runtime_instance_id,
                });
            }
            let snapshot_branch_id = admitted_snapshot.snapshot().meta.branch_id;
            if snapshot_branch_id != branch_id {
                return Err(
                    SignalBranchRetirementDenial::RetirementSnapshotBranchMismatch {
                        branch_id,
                        snapshot_branch_id,
                    },
                );
            }
            unique_retentions.insert(admitted_snapshot.retention_identity());
        }
        Ok(1_u32.saturating_add(unique_retentions.len() as u32))
    }
}
