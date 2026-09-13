use std::sync::Arc;

use crate::branch::retention::SignalBranchAdmissionReservation;
use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchForkOperationDenial, ValidatedSignalBranchName,
};
use crate::state::SignalBranchHandle;

use super::super::branch_registry::SignalBranchOwnedReservation;
use super::super::owner_metadata::SignalOwnerOwnedForkLineageReservation;
use super::super::{SignalBranchCellState, SignalOwnerOperationAdmission};
use super::SignalOwner;

pub(in crate::branch::owner_services) struct SignalOwnedForkDestination<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) handle: SignalBranchHandle,
    pub(in crate::branch::owner_services) owner_runtime_instance_id: u64,
    pub(in crate::branch::owner_services) definition_basis: u64,
    pub(in crate::branch::owner_services) registry:
        SignalBranchOwnedReservation<SignalBranchCellState<D, I, T>>,
    pub(in crate::branch::owner_services) lineage: SignalOwnerOwnedForkLineageReservation<D, I, T>,
    pub(in crate::branch::owner_services) retention: SignalBranchAdmissionReservation,
}

impl<D, I, T> SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn reserve_fork_destination_owned(
        self: &Arc<Self>,
        admission: &SignalOwnerOperationAdmission<'_>,
        source: &AdmittedSignalBranchBasis,
        requested_identity: ValidatedSignalBranchName,
    ) -> Result<SignalOwnedForkDestination<D, I, T>, SignalBranchForkOperationDenial> {
        let handle = self.issue_fork_destination_identity(admission, source, requested_identity)?;
        let branch_id = handle.id;
        let parent_branch_id = source.owner_branch_id();
        let registry = self
            .registry
            .reserve_named_owned(admission, branch_id, handle.name.clone())
            .map_err(|denial| {
                super::fork_denials::map_fork_registry_denial(denial, parent_branch_id)
            })?;
        let lineage = self.reserve_fork_child_owned(admission, parent_branch_id, branch_id)?;
        let retention = self
            .reserve_admitted_retention(admission, branch_id, 1)
            .map_err(|denial| SignalBranchForkOperationDenial::RetentionUnavailable { denial })?;
        Ok(SignalOwnedForkDestination {
            handle,
            owner_runtime_instance_id: self.runtime_instance_id,
            definition_basis: self.definition_basis,
            registry,
            lineage,
            retention,
        })
    }
}
