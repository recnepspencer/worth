use std::collections::BTreeSet;

use crate::branch::{
    PlannedSignalBranchRetirement, SignalBranchRetirementDenial, SignalBranchRetirementReceipt,
};
use crate::state::SignalBranchId;

use super::super::owner_metadata::SignalOwnerRetirementMetadataReservation;
use super::super::{
    SignalBranchCellState, SignalBranchRegistryDenial, SignalBranchRetirement,
    SignalOwnerCancellationToken, SignalOwnerOperationAdmission, SignalOwnerUnavailable,
};
use super::SignalOwner;

pub(in crate::branch::owner_services) struct SignalOwnerRetirementReservation<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    metadata: Option<SignalOwnerRetirementMetadataReservation<'a, D, I, T>>,
    retirement: Option<SignalBranchRetirement<'a, SignalBranchCellState<D, I, T>>>,
}

impl<D, I, T> SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn reserve_retirement<'a>(
        &'a self,
        admission: &'a SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<SignalOwnerRetirementReservation<'a, D, I, T>, SignalBranchRetirementDenial> {
        if branch_id == self.selected_branch_id() {
            return Err(SignalBranchRetirementDenial::CurrentBranch { branch_id });
        }
        let metadata = self.metadata.reserve_retirement(admission, branch_id)?;
        let retention_counts = self.retirement_retention_counts(branch_id);
        if retention_counts.external != 0 {
            return Err(SignalBranchRetirementDenial::RetainedComponentBasis {
                branch_id,
                active_leases: retention_counts.external,
            });
        }
        if retention_counts.admitted_or_reserved != 1 {
            return Err(SignalBranchRetirementDenial::RetainedAdmittedBasis {
                branch_id,
                active_leases: retention_counts.admitted_or_reserved,
            });
        }
        let retirement = self
            .begin_retirement(admission, branch_id)
            .map_err(|denial| map_retirement_registry_denial(denial, branch_id))?;
        Ok(SignalOwnerRetirementReservation {
            metadata: Some(metadata),
            retirement: Some(retirement),
        })
    }

    pub(in crate::branch::owner_services) fn reserve_batch_retirement_after<'a>(
        &'a self,
        admission: &'a SignalOwnerOperationAdmission<'_>,
        plan: &PlannedSignalBranchRetirement,
        retired_before: &BTreeSet<SignalBranchId>,
    ) -> Result<SignalOwnerRetirementReservation<'a, D, I, T>, SignalBranchRetirementDenial> {
        if !self.basis_has_owner_affinity(plan.admitted_basis()) {
            return Err(SignalBranchRetirementDenial::CanonicalBasisMismatch);
        }
        let branch_id = plan.branch().id;
        let retirement = self
            .begin_retirement(admission, branch_id)
            .map_err(|denial| map_retirement_registry_denial(denial, branch_id))?;
        let cell = retirement
            .preflight_exact_reserved()
            .map_err(|denial| map_retirement_registry_denial(denial, branch_id))??;
        if cell
            .observation
            .compare(plan.admitted_basis().observation())
            .is_err()
        {
            return Err(SignalBranchRetirementDenial::StaleBranchHead {
                expected_generation: plan.admitted_basis().observation().generation().get(),
                observed_generation: cell.observation.generation().get(),
            });
        }
        if branch_id == self.selected_branch_id() {
            return Err(SignalBranchRetirementDenial::CurrentBranch { branch_id });
        }
        if cell.branch.parent_branch_id.is_none() {
            return Err(SignalBranchRetirementDenial::CanonicalBranch { branch_id });
        }
        let metadata =
            self.metadata
                .reserve_retirement_after(admission, branch_id, retired_before)?;
        let retention_counts = self.retirement_retention_counts(branch_id);
        if retention_counts.external != 0 {
            return Err(SignalBranchRetirementDenial::RetainedComponentBasis {
                branch_id,
                active_leases: retention_counts.external,
            });
        }
        if retention_counts.admitted_or_reserved != 1 {
            return Err(SignalBranchRetirementDenial::RetainedAdmittedBasis {
                branch_id,
                active_leases: retention_counts.admitted_or_reserved,
            });
        }
        let shared_holders = plan.admitted_basis().shared_holder_count();
        if shared_holders != 1 {
            return Err(SignalBranchRetirementDenial::SharedAdmittedBasis {
                branch_id,
                shared_holders,
            });
        }
        Ok(SignalOwnerRetirementReservation {
            metadata: Some(metadata),
            retirement: Some(retirement),
        })
    }
}

impl<D, I, T> SignalOwnerRetirementReservation<'_, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn execute(
        mut self,
        plan: PlannedSignalBranchRetirement,
        cancellation: &SignalOwnerCancellationToken,
    ) -> Result<SignalBranchRetirementReceipt, SignalBranchRetirementDenial> {
        self.execute_with_observers(
            plan,
            cancellation,
            || {},
            || {},
            |cleanup| cleanup.discard(),
        )
    }

    fn execute_with_observers(
        &mut self,
        plan: PlannedSignalBranchRetirement,
        cancellation: &SignalOwnerCancellationToken,
        before_movement: impl FnOnce(),
        after_movement: impl FnOnce(),
        discard_cleanup: impl FnOnce(crate::logic::transaction::SignalOwnerRetirementCleanup<D, I, T>),
    ) -> Result<SignalBranchRetirementReceipt, SignalBranchRetirementDenial> {
        let branch_id = plan.branch().id;
        let shared_holders = plan.admitted_basis().shared_holder_count();
        if shared_holders != 1 {
            return Err(SignalBranchRetirementDenial::SharedAdmittedBasis {
                branch_id,
                shared_holders,
            });
        }
        let snapshot_count = self
            .metadata
            .as_ref()
            .expect("a live retirement reservation retains metadata capacity")
            .snapshot_count();
        let retirement = self
            .retirement
            .as_mut()
            .expect("a live retirement reservation retains registry membership");
        let branch_id = retirement.branch_id();
        let outcome = retirement
            .execute_exact_reserved(
                plan,
                cancellation,
                snapshot_count,
                before_movement,
                after_movement,
            )
            .map_err(|denial| map_retirement_registry_denial(denial, branch_id))??;
        let receipt = outcome.receipt;
        let parent_branch_id = Some(receipt.parent_branch_id());
        let cleanup = self
            .metadata
            .take()
            .expect("performed retirement retains its receipt reservation")
            .complete(parent_branch_id, receipt.clone());
        debug_assert_eq!(cleanup.reclaimed_snapshot_count(), snapshot_count);
        discard_cleanup(cleanup);
        let _ = retirement.recover_performed_receipt();
        Ok(receipt)
    }

    #[cfg(test)]
    pub(in crate::branch::owner_services) fn execute_with_post_movement_fault(
        mut self,
        plan: PlannedSignalBranchRetirement,
        cancellation: &SignalOwnerCancellationToken,
    ) -> Result<SignalBranchRetirementReceipt, SignalBranchRetirementDenial> {
        self.execute_with_observers(
            plan,
            cancellation,
            || {},
            || panic!("inject owner retirement post-movement fault"),
            |cleanup| cleanup.discard(),
        )
    }

    #[cfg(test)]
    pub(in crate::branch::owner_services) fn execute_with_cleanup_observer(
        mut self,
        plan: PlannedSignalBranchRetirement,
        cancellation: &SignalOwnerCancellationToken,
        before_payload_drop: impl FnOnce(u32),
    ) -> Result<SignalBranchRetirementReceipt, SignalBranchRetirementDenial> {
        self.execute_with_observers(
            plan,
            cancellation,
            || {},
            || {},
            |cleanup| cleanup.discard_with_observer(before_payload_drop),
        )
    }

    #[cfg(test)]
    pub(in crate::branch::owner_services) fn execute_with_cancellation_observers(
        mut self,
        plan: PlannedSignalBranchRetirement,
        cancellation: &SignalOwnerCancellationToken,
        before_movement: impl FnOnce(),
        after_movement: impl FnOnce(),
    ) -> Result<SignalBranchRetirementReceipt, SignalBranchRetirementDenial> {
        self.execute_with_observers(
            plan,
            cancellation,
            before_movement,
            after_movement,
            |cleanup| cleanup.discard(),
        )
    }
}

impl<D, I, T> Drop for SignalOwnerRetirementReservation<'_, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn drop(&mut self) {
        let Some(retirement) = self.retirement.as_mut() else {
            return;
        };
        let Some(receipt) = retirement.recover_performed_receipt() else {
            return;
        };
        let parent_branch_id = Some(receipt.parent_branch_id());
        if let Some(metadata) = self.metadata.take() {
            metadata.complete(parent_branch_id, receipt).discard();
        }
    }
}

pub(in crate::branch::owner_services) fn map_retirement_registry_denial(
    denial: SignalBranchRegistryDenial,
    branch_id: SignalBranchId,
) -> SignalBranchRetirementDenial {
    match denial {
        SignalBranchRegistryDenial::ForeignOwner | SignalBranchRegistryDenial::ExpiredAdmission => {
            SignalBranchRetirementDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalBranchRegistryDenial::UnknownBranch(_) => {
            SignalBranchRetirementDenial::UnknownBranch { branch_id }
        }
        SignalBranchRegistryDenial::RetirementInProgress(_) => {
            SignalBranchRetirementDenial::RetirementInProgress { branch_id }
        }
        SignalBranchRegistryDenial::OwnerReentry => SignalBranchRetirementDenial::OwnerReentry,
        SignalBranchRegistryDenial::OwnerMetadataOrdering => {
            SignalBranchRetirementDenial::OwnerCellMisuse { branch_id }
        }
        SignalBranchRegistryDenial::TargetCellDenied(denial) => {
            super::super::branch_execution_cell::retirement::map_retirement_cell_denial(
                denial, branch_id,
            )
        }
        SignalBranchRegistryDenial::DuplicateBranch(_)
        | SignalBranchRegistryDenial::LiveCapacityExhausted { .. }
        | SignalBranchRegistryDenial::ReservationCapacityExhausted { .. }
        | SignalBranchRegistryDenial::NameAlreadyReserved
        | SignalBranchRegistryDenial::NameAlreadyInstalled
        | SignalBranchRegistryDenial::ExpiredRetirement(_) => {
            SignalBranchRetirementDenial::OwnerInvariantViolation { branch_id }
        }
    }
}
