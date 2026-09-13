use std::collections::BTreeSet;

use worth_proof::TransitionOutcome;

use crate::branch::{
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot, PlannedSignalBranchRetirementBatch,
    SignalBranchRetirementBatchDenial, SignalBranchRetirementReason,
};
use crate::state::SignalBranchHandle;

use super::super::SignalOwnerOperationAdmission;
use super::SignalOwner;

impl<D, I, T> SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn plan_legacy_retirement_batch(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
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
            let plan = match self.plan_legacy_retirement_after(
                admission, branch, basis, reason, allowance, &scheduled,
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
}
