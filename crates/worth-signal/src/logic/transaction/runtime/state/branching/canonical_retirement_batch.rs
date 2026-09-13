use std::collections::BTreeSet;

use worth_proof::TransitionOutcome;

use crate::branch::{AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot};
use crate::state::SignalBranchHandle;

use super::super::runtime_state::SignalRuntime;
use super::{
    PlannedSignalBranchRetirementBatch, SignalBranchRetirementBatchDenial,
    SignalBranchRetirementBatchReceipt, SignalBranchRetirementDenial, SignalBranchRetirementReason,
    SignalBranchRetirementRequest,
};

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Plan an atomic retirement batch from one unique admitted basis per
    /// branch. Child-before-parent ordering is carried in `scheduled`.
    pub fn plan_signal_branch_retirement_batch(
        &mut self,
        requests: Vec<(
            SignalBranchHandle,
            AdmittedSignalBranchBasis,
            SignalBranchRetirementReason,
        )>,
    ) -> TransitionOutcome<PlannedSignalBranchRetirementBatch, SignalBranchRetirementBatchDenial>
    {
        if self.owner_services.is_sealed() {
            return self.owner_services.plan_legacy_retirement_batch(requests);
        }
        if requests.is_empty() {
            return TransitionOutcome::denied(SignalBranchRetirementBatchDenial::Empty);
        }
        self.plan_signal_branch_retirement_batch_with_snapshot_releases(
            requests
                .into_iter()
                .map(|(branch, basis, reason)| (branch, basis, Vec::new(), reason))
                .collect(),
        )
    }

    /// Plan an atomic retirement batch while accounting for concrete admitted
    /// snapshots that will be released before execution.
    pub fn plan_signal_branch_retirement_batch_releasing_snapshots(
        &mut self,
        requests: Vec<(
            SignalBranchHandle,
            AdmittedSignalBranchBasis,
            Vec<&AdmittedSignalBranchSnapshot>,
            SignalBranchRetirementReason,
        )>,
    ) -> TransitionOutcome<PlannedSignalBranchRetirementBatch, SignalBranchRetirementBatchDenial>
    {
        if self.owner_services.is_sealed() {
            return self
                .owner_services
                .plan_legacy_retirement_batch_releasing_snapshots(requests);
        }
        self.plan_signal_branch_retirement_batch_with_snapshot_releases(requests)
    }

    fn plan_signal_branch_retirement_batch_with_snapshot_releases(
        &mut self,
        requests: Vec<(
            SignalBranchHandle,
            AdmittedSignalBranchBasis,
            Vec<&AdmittedSignalBranchSnapshot>,
            SignalBranchRetirementReason,
        )>,
    ) -> TransitionOutcome<PlannedSignalBranchRetirementBatch, SignalBranchRetirementBatchDenial>
    {
        if requests.is_empty() {
            return TransitionOutcome::denied(SignalBranchRetirementBatchDenial::Empty);
        }
        let mut unique = BTreeSet::new();
        for (branch, ..) in &requests {
            if !unique.insert(branch.id) {
                return TransitionOutcome::denied(
                    SignalBranchRetirementBatchDenial::DuplicateBranch {
                        branch_id: branch.id,
                    },
                );
            }
        }
        let mut scheduled = BTreeSet::new();
        let mut plans = Vec::with_capacity(requests.len());
        for (position, (branch, basis, releasing_snapshots, reason)) in
            requests.into_iter().enumerate()
        {
            let branch_id = branch.id;
            let allowance =
                match self.retirement_snapshot_allowance(branch_id, &releasing_snapshots) {
                    Ok(allowance) => allowance,
                    Err(denial) => {
                        return TransitionOutcome::denied(
                            SignalBranchRetirementBatchDenial::Retirement {
                                position: position as u32,
                                denial,
                            },
                        )
                    }
                };
            let Some(branch) = self.branches.branch_handle(branch_id) else {
                return TransitionOutcome::denied(SignalBranchRetirementBatchDenial::Retirement {
                    position: position as u32,
                    denial: SignalBranchRetirementDenial::UnknownBranch { branch_id },
                });
            };
            let Ok(live) = self.signal_branch_observation(&branch) else {
                return TransitionOutcome::denied(SignalBranchRetirementBatchDenial::Retirement {
                    position: position as u32,
                    denial: SignalBranchRetirementDenial::UnknownBranch { branch_id },
                });
            };
            if live.compare(basis.observation()).is_err() {
                return TransitionOutcome::denied(SignalBranchRetirementBatchDenial::Retirement {
                    position: position as u32,
                    denial: SignalBranchRetirementDenial::CanonicalBasisMismatch,
                });
            }
            let head = match self.branch_transaction_head(branch.clone()) {
                TransitionOutcome::Success(head) => head,
                TransitionOutcome::Denied(_) | TransitionOutcome::Failed(_) => {
                    return TransitionOutcome::denied(
                        SignalBranchRetirementBatchDenial::Retirement {
                            position: position as u32,
                            denial: SignalBranchRetirementDenial::UnknownBranch {
                                branch_id: branch.id,
                            },
                        },
                    )
                }
            };
            let request = SignalBranchRetirementRequest::new(branch, head, reason);
            let plan = match self.plan_branch_retirement_with_canonical_basis_after(
                request, basis, &scheduled, allowance,
            ) {
                TransitionOutcome::Success(plan) => plan,
                TransitionOutcome::Denied(denial) => {
                    return TransitionOutcome::denied(
                        SignalBranchRetirementBatchDenial::Retirement {
                            position: position as u32,
                            denial,
                        },
                    )
                }
                TransitionOutcome::Failed(_) => {
                    unreachable!("Signal retirement planning has no failure lane")
                }
            };
            plans.push(plan);
            scheduled.insert(branch_id);
        }
        TransitionOutcome::success(PlannedSignalBranchRetirementBatch { plans })
    }

    pub fn retire_signal_branch_batch(
        &mut self,
        plan: PlannedSignalBranchRetirementBatch,
    ) -> TransitionOutcome<SignalBranchRetirementBatchReceipt, SignalBranchRetirementBatchDenial>
    {
        if self.owner_services.is_sealed() {
            return self.owner_services.retire_legacy_batch(plan);
        }
        self.retire_branch_batch(plan)
    }
}
