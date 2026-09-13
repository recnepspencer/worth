use std::collections::BTreeSet;

use crate::branch::{SignalBranchRetirementDenial, SignalBranchRetirementReceipt};
use crate::state::SignalBranchId;

use super::super::lifecycle_state::SignalOwnerMetadataHoldDenial;
use super::super::{SignalOwnerOperationAdmission, SignalOwnerUnavailable};
use super::SignalOwnerMetadata;
#[cfg(test)]
use super::SignalOwnerMetadataAuthorizationDenial;

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::branch::owner_services) struct SignalOwnerRetirementContractObservation {
    pub(crate) active_reservations: usize,
    pub(crate) reserved_receipt_count: usize,
    pub(crate) retained_receipt_count: usize,
}

impl<D, I, T> SignalOwnerMetadata<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn reserve_retirement<'a>(
        &'a self,
        admission: &'a SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<SignalOwnerRetirementMetadataReservation<'a, D, I, T>, SignalBranchRetirementDenial>
    {
        self.reserve_retirement_after(admission, branch_id, &BTreeSet::new())
    }

    pub(in crate::branch::owner_services) fn reserve_retirement_after<'a>(
        &'a self,
        admission: &'a SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
        retired_before: &BTreeSet<SignalBranchId>,
    ) -> Result<SignalOwnerRetirementMetadataReservation<'a, D, I, T>, SignalBranchRetirementDenial>
    {
        admission
            .authorize(self.runtime_instance_id, self.lifecycle_identity)
            .map_err(|_| SignalBranchRetirementDenial::OwnerUnavailable(SignalOwnerUnavailable))?;
        let _hold = admission
            .hold_owner_metadata()
            .map_err(|denial| match denial {
                SignalOwnerMetadataHoldDenial::BranchCellOrMetadataAlreadyHeld => {
                    SignalBranchRetirementDenial::OwnerCellMisuse { branch_id }
                }
                SignalOwnerMetadataHoldDenial::ExecutingThreadReentry => {
                    SignalBranchRetirementDenial::OwnerReentry
                }
            })?;
        let snapshot_count = self
            .lock()
            .reserve_retirement_contract_after(branch_id, retired_before)?;
        Ok(SignalOwnerRetirementMetadataReservation {
            metadata: self,
            admission,
            branch_id,
            snapshot_count,
            completed: false,
        })
    }

    #[cfg(test)]
    pub(in crate::branch::owner_services) fn retirement_contract_observation(
        &self,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
    ) -> Result<SignalOwnerRetirementContractObservation, SignalBranchRetirementDenial> {
        let _hold = self.authorize(admission).map_err(|denial| match denial {
            SignalOwnerMetadataAuthorizationDenial::OwnerUnavailable => {
                SignalBranchRetirementDenial::OwnerUnavailable(SignalOwnerUnavailable)
            }
            SignalOwnerMetadataAuthorizationDenial::OwnerCellMisuse => {
                SignalBranchRetirementDenial::OwnerCellMisuse { branch_id }
            }
            SignalOwnerMetadataAuthorizationDenial::OwnerReentry => {
                SignalBranchRetirementDenial::OwnerReentry
            }
        })?;
        let (active_reservations, reserved_receipt_count, retained_receipt_count) =
            self.lock().retirement_contract_counts();
        Ok(SignalOwnerRetirementContractObservation {
            active_reservations,
            reserved_receipt_count,
            retained_receipt_count,
        })
    }
}

pub(in crate::branch::owner_services) struct SignalOwnerRetirementMetadataReservation<'a, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    metadata: &'a SignalOwnerMetadata<D, I, T>,
    admission: &'a SignalOwnerOperationAdmission<'a>,
    branch_id: SignalBranchId,
    snapshot_count: u32,
    completed: bool,
}

impl<D, I, T> SignalOwnerRetirementMetadataReservation<'_, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) const fn snapshot_count(&self) -> u32 {
        self.snapshot_count
    }

    pub(in crate::branch::owner_services) fn complete(
        mut self,
        parent_branch_id: Option<SignalBranchId>,
        receipt: SignalBranchRetirementReceipt,
    ) -> crate::logic::transaction::SignalOwnerRetirementCleanup<D, I, T> {
        debug_assert!(self.admission.permits_owner_lock_acquisition());
        let cleanup = {
            self.metadata.lock().complete_retirement_contract(
                self.branch_id,
                parent_branch_id,
                receipt,
            )
        };
        self.completed = true;
        cleanup
    }
}

impl<D, I, T> Drop for SignalOwnerRetirementMetadataReservation<'_, D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn drop(&mut self) {
        if !self.completed {
            debug_assert!(self.admission.permits_owner_lock_acquisition());
            self.metadata
                .lock()
                .cancel_retirement_contract(self.branch_id);
        }
    }
}
