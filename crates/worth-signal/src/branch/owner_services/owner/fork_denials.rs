use crate::branch::{SignalBranchForkOperationDenial, SignalOwnerUnavailable};
use crate::data::error::SignalError;
use crate::state::SignalBranchId;

use super::super::branch_registry::SignalBranchRegistryDenial;
use super::super::lifecycle_state::SignalOwnerMetadataHoldDenial;

pub(in crate::branch::owner_services) fn map_fork_registry_denial(
    denial: SignalBranchRegistryDenial,
    branch_id: SignalBranchId,
) -> SignalBranchForkOperationDenial {
    match denial {
        SignalBranchRegistryDenial::ForeignOwner | SignalBranchRegistryDenial::ExpiredAdmission => {
            SignalBranchForkOperationDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalBranchRegistryDenial::LiveCapacityExhausted {
            maximum_live_branches,
        } => SignalBranchForkOperationDenial::LiveBranchCapacityExhausted {
            maximum_live_branches,
        },
        SignalBranchRegistryDenial::ReservationCapacityExhausted {
            maximum_reservations,
        } => SignalBranchForkOperationDenial::ReservationCapacityExhausted {
            maximum_reservations,
        },
        SignalBranchRegistryDenial::NameAlreadyReserved => {
            SignalBranchForkOperationDenial::NameAlreadyReserved
        }
        SignalBranchRegistryDenial::NameAlreadyInstalled => {
            SignalBranchForkOperationDenial::NameAlreadyInstalled
        }
        SignalBranchRegistryDenial::OwnerReentry => SignalBranchForkOperationDenial::OwnerReentry,
        SignalBranchRegistryDenial::OwnerMetadataOrdering => {
            SignalBranchForkOperationDenial::OwnerCellMisuse { branch_id }
        }
        denial => SignalBranchForkOperationDenial::OwnerDeniedNoMovement {
            error: SignalError::internal(format!(
                "Signal owner fork reservation invariant failed: {denial:?}"
            )),
        },
    }
}

pub(in crate::branch::owner_services) fn map_fork_owner_lock_denial(
    denial: SignalOwnerMetadataHoldDenial,
    branch_id: SignalBranchId,
) -> SignalBranchForkOperationDenial {
    match denial {
        SignalOwnerMetadataHoldDenial::BranchCellOrMetadataAlreadyHeld => {
            SignalBranchForkOperationDenial::OwnerCellMisuse { branch_id }
        }
        SignalOwnerMetadataHoldDenial::ExecutingThreadReentry => {
            SignalBranchForkOperationDenial::OwnerReentry
        }
    }
}
