use std::collections::BTreeSet;

use worth_proof::TransitionOutcome;

use crate::branch::{
    PlannedSignalBranchRetirementBatch, SignalBranchRetirementBatchDenial,
    SignalBranchRetirementBatchReceipt,
};

use super::super::{SignalOwnerCancellationToken, SignalOwnerOperationAdmission};
use super::SignalOwner;

impl<D, I, T> SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn retire_legacy_batch(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        batch: PlannedSignalBranchRetirementBatch,
        cancellation: &SignalOwnerCancellationToken,
    ) -> TransitionOutcome<SignalBranchRetirementBatchReceipt, SignalBranchRetirementBatchDenial>
    {
        let mut scheduled = BTreeSet::new();
        let mut reservations = Vec::with_capacity(batch.plans.len());
        for (position, plan) in batch.plans.iter().enumerate() {
            let branch_id = plan.branch().id;
            if scheduled.contains(&branch_id) {
                return TransitionOutcome::denied(
                    SignalBranchRetirementBatchDenial::DuplicateBranch { branch_id },
                );
            }
            let reservation = match self.reserve_batch_retirement_after(admission, plan, &scheduled)
            {
                Ok(reservation) => reservation,
                Err(denial) => {
                    return TransitionOutcome::denied(
                        SignalBranchRetirementBatchDenial::Retirement {
                            position: position as u32,
                            denial,
                        },
                    )
                }
            };
            reservations.push(reservation);
            scheduled.insert(branch_id);
        }

        let mut receipts = Vec::with_capacity(batch.plans.len());
        for (position, (reservation, plan)) in reservations
            .into_iter()
            .zip(batch.plans.into_iter())
            .enumerate()
        {
            match reservation.execute(plan, cancellation) {
                Ok(receipt) => receipts.push(receipt),
                Err(denial) => {
                    return TransitionOutcome::denied(
                        SignalBranchRetirementBatchDenial::Retirement {
                            position: position as u32,
                            denial,
                        },
                    )
                }
            }
        }
        TransitionOutcome::success(SignalBranchRetirementBatchReceipt::new(receipts))
    }
}
