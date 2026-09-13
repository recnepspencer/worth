use std::sync::Arc;

use crate::state::SignalBranchId;

use super::super::lifecycle_state::SignalOwnerOperationAdmission;
use super::{
    SignalBranchNameOccupancy, SignalBranchOwnedReservation, SignalBranchRegistry,
    SignalBranchRegistryDenial, SignalBranchRegistryState, SignalBranchReservation,
};

impl<S> SignalBranchRegistry<S> {
    pub(crate) fn reserve_named<'a>(
        &'a self,
        admission: &'a SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
        name: String,
    ) -> Result<SignalBranchReservation<'a, S>, SignalBranchRegistryDenial> {
        self.reserve_entry(admission, branch_id, Some(&name))?;
        Ok(SignalBranchReservation::named(
            self, admission, branch_id, name,
        ))
    }

    pub(crate) fn reserve_named_owned(
        self: &Arc<Self>,
        admission: &SignalOwnerOperationAdmission<'_>,
        branch_id: SignalBranchId,
        name: String,
    ) -> Result<SignalBranchOwnedReservation<S>, SignalBranchRegistryDenial> {
        self.reserve_entry(admission, branch_id, Some(&name))?;
        Ok(SignalBranchOwnedReservation::new(
            Arc::clone(self),
            branch_id,
            name,
        ))
    }
}

pub(super) fn mark_name_installed<S>(
    state: &mut SignalBranchRegistryState<S>,
    branch_id: SignalBranchId,
    name: &str,
) -> Result<(), SignalBranchRegistryDenial> {
    match state.names.get(name) {
        Some(SignalBranchNameOccupancy::Reserved(reserved_branch_id))
            if *reserved_branch_id == branch_id =>
        {
            state.names.insert(
                name.to_owned(),
                SignalBranchNameOccupancy::Installed(branch_id),
            );
            Ok(())
        }
        Some(SignalBranchNameOccupancy::Reserved(_)) => {
            Err(SignalBranchRegistryDenial::NameAlreadyReserved)
        }
        Some(SignalBranchNameOccupancy::Installed(_)) => {
            Err(SignalBranchRegistryDenial::NameAlreadyInstalled)
        }
        None => Err(SignalBranchRegistryDenial::UnknownBranch(branch_id)),
    }
}

pub(super) fn remove_name_for_branch<S>(
    state: &mut SignalBranchRegistryState<S>,
    branch_id: SignalBranchId,
) {
    let Some(name) = state.names_by_branch.remove(&branch_id) else {
        return;
    };
    let owned_by_branch = matches!(
        state.names.get(&name),
        Some(
            SignalBranchNameOccupancy::Reserved(owner_branch_id)
                | SignalBranchNameOccupancy::Installed(owner_branch_id)
        ) if *owner_branch_id == branch_id
    );
    if owned_by_branch {
        state.names.remove(&name);
    }
}
