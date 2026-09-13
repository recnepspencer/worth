use std::collections::BTreeSet;
use worth_proof::TransitionOutcome;

use super::super::runtime_state::SignalRuntime;
use super::{
    PlannedSignalBranchRetirementBatch, SignalBranchRetirementBatchDenial,
    SignalBranchRetirementBatchReceipt, SignalBranchRetirementDenial,
    SignalBranchRetirementReceipt, SignalBranchRetirementRequest,
};
use crate::state::SignalBranchId;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn retire_branch_batch(
        &mut self,
        plan: PlannedSignalBranchRetirementBatch,
    ) -> TransitionOutcome<SignalBranchRetirementBatchReceipt, SignalBranchRetirementBatchDenial>
    {
        let mut scheduled = BTreeSet::new();
        for (position, retirement) in plan.plans.iter().enumerate() {
            let branch = retirement.branch();
            let live = match self.signal_branch_observation(branch) {
                Ok(live) => live,
                Err(_) => {
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
            if live
                .compare(retirement.admitted_basis().observation())
                .is_err()
            {
                return TransitionOutcome::denied(SignalBranchRetirementBatchDenial::Retirement {
                    position: position as u32,
                    denial: SignalBranchRetirementDenial::StaleBranchHead {
                        expected_generation: retirement
                            .admitted_basis()
                            .observation()
                            .generation()
                            .get(),
                        observed_generation: live.generation().get(),
                    },
                });
            }
            let head = match self.branch_transaction_head(branch.clone()) {
                TransitionOutcome::Success(head) => head,
                _ => {
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
            let request =
                SignalBranchRetirementRequest::new(branch.clone(), head, retirement.reason());
            if let Err(denial) =
                self.validate_retirement_request_after_with_admitted(&request, &scheduled, 1)
            {
                return TransitionOutcome::denied(SignalBranchRetirementBatchDenial::Retirement {
                    position: position as u32,
                    denial,
                });
            }
            scheduled.insert(branch.id);
        }
        let mut receipts = Vec::with_capacity(plan.plans.len());
        for retirement in plan.plans {
            match self.retire_branch(retirement) {
                TransitionOutcome::Success(receipt) => receipts.push(receipt),
                other => panic!("prevalidated retirement batch must execute atomically: {other:?}"),
            }
        }
        TransitionOutcome::success(SignalBranchRetirementBatchReceipt::new(receipts))
    }

    pub fn branch_retirement_receipt(
        &self,
        branch_id: SignalBranchId,
    ) -> Option<SignalBranchRetirementReceipt> {
        if self.owner_services.is_sealed() {
            return self.owner_services.legacy_retirement_receipt(branch_id);
        }
        self.branches.branch_retirement_receipt(branch_id).cloned()
    }
}
