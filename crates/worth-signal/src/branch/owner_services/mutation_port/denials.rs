use crate::branch::{
    SignalBranchAdvanceDenial, SignalBranchForkOperationDenial, SignalBranchRestoreDenial,
    SignalBranchSnapshotCaptureDenial,
};
use crate::data::error::SignalError;
use crate::state::SignalBranchId;

use super::super::branch_execution_cell::{advance, fork, restoration, snapshot};
use super::super::{
    SignalBranchRegistryDenial, SignalOwnerAdmissionDenial, SignalOwnerUnavailable,
};

macro_rules! map_admission_denial {
    ($function:ident, $output:ident) => {
        pub(super) fn $function(denial: SignalOwnerAdmissionDenial) -> $output {
            match denial {
                SignalOwnerAdmissionDenial::ForeignOwner
                | SignalOwnerAdmissionDenial::OwnerUnavailable => {
                    $output::OwnerUnavailable(SignalOwnerUnavailable)
                }
                SignalOwnerAdmissionDenial::OperationCapacityExhausted {
                    maximum_in_flight_operations,
                } => $output::OperationCapacityExhausted {
                    maximum_in_flight_operations,
                },
                SignalOwnerAdmissionDenial::OwnerReentry => $output::OwnerReentry,
            }
        }
    };
}

map_admission_denial!(map_fork_admission_denial, SignalBranchForkOperationDenial);
map_admission_denial!(map_advance_admission_denial, SignalBranchAdvanceDenial);
map_admission_denial!(
    map_capture_admission_denial,
    SignalBranchSnapshotCaptureDenial
);
map_admission_denial!(map_restore_admission_denial, SignalBranchRestoreDenial);

pub(super) fn map_fork_registry_denial(
    denial: SignalBranchRegistryDenial,
    branch_id: SignalBranchId,
) -> SignalBranchForkOperationDenial {
    match denial {
        SignalBranchRegistryDenial::ForeignOwner | SignalBranchRegistryDenial::ExpiredAdmission => {
            SignalBranchForkOperationDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalBranchRegistryDenial::UnknownBranch(_) => {
            SignalBranchForkOperationDenial::UnknownBranch { branch_id }
        }
        SignalBranchRegistryDenial::RetirementInProgress(_) => {
            SignalBranchForkOperationDenial::RetirementInProgress { branch_id }
        }
        SignalBranchRegistryDenial::NameAlreadyReserved => {
            SignalBranchForkOperationDenial::NameAlreadyReserved
        }
        SignalBranchRegistryDenial::NameAlreadyInstalled => {
            SignalBranchForkOperationDenial::NameAlreadyInstalled
        }
        SignalBranchRegistryDenial::TargetCellDenied(denial) => {
            fork::map_fork_cell_denial(denial, branch_id)
        }
        SignalBranchRegistryDenial::OwnerMetadataOrdering => {
            SignalBranchForkOperationDenial::OwnerCellMisuse { branch_id }
        }
        SignalBranchRegistryDenial::OwnerReentry => SignalBranchForkOperationDenial::OwnerReentry,
        denial => SignalBranchForkOperationDenial::OwnerDeniedNoMovement {
            error: registry_invariant_error("fork lookup", denial),
        },
    }
}

pub(super) fn map_advance_registry_denial(
    denial: SignalBranchRegistryDenial,
    branch_id: SignalBranchId,
) -> SignalBranchAdvanceDenial {
    match denial {
        SignalBranchRegistryDenial::ForeignOwner | SignalBranchRegistryDenial::ExpiredAdmission => {
            SignalBranchAdvanceDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalBranchRegistryDenial::UnknownBranch(_) => {
            SignalBranchAdvanceDenial::UnknownBranch { branch_id }
        }
        SignalBranchRegistryDenial::RetirementInProgress(_) => {
            SignalBranchAdvanceDenial::RetirementInProgress { branch_id }
        }
        SignalBranchRegistryDenial::TargetCellDenied(denial) => {
            advance::map_advance_cell_denial(denial, branch_id)
        }
        SignalBranchRegistryDenial::OwnerMetadataOrdering => {
            SignalBranchAdvanceDenial::OwnerCellMisuse { branch_id }
        }
        SignalBranchRegistryDenial::OwnerReentry => SignalBranchAdvanceDenial::OwnerReentry,
        denial => SignalBranchAdvanceDenial::MutationFailedNoMovement {
            error: registry_invariant_error("advance lookup", denial),
        },
    }
}

pub(super) fn map_capture_registry_denial(
    denial: SignalBranchRegistryDenial,
    branch_id: SignalBranchId,
) -> SignalBranchSnapshotCaptureDenial {
    match denial {
        SignalBranchRegistryDenial::ForeignOwner | SignalBranchRegistryDenial::ExpiredAdmission => {
            SignalBranchSnapshotCaptureDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalBranchRegistryDenial::UnknownBranch(_) => {
            SignalBranchSnapshotCaptureDenial::UnknownBranch { branch_id }
        }
        SignalBranchRegistryDenial::RetirementInProgress(_) => {
            SignalBranchSnapshotCaptureDenial::RetirementInProgress { branch_id }
        }
        SignalBranchRegistryDenial::TargetCellDenied(denial) => {
            snapshot::map_snapshot_cell_denial(denial, branch_id)
        }
        SignalBranchRegistryDenial::OwnerMetadataOrdering => {
            SignalBranchSnapshotCaptureDenial::OwnerCellMisuse { branch_id }
        }
        SignalBranchRegistryDenial::OwnerReentry => SignalBranchSnapshotCaptureDenial::OwnerReentry,
        denial => SignalBranchSnapshotCaptureDenial::OwnerDeniedNoMovement {
            error: registry_invariant_error("snapshot lookup", denial),
        },
    }
}

pub(super) fn map_restore_registry_denial(
    denial: SignalBranchRegistryDenial,
    branch_id: SignalBranchId,
) -> SignalBranchRestoreDenial {
    match denial {
        SignalBranchRegistryDenial::ForeignOwner | SignalBranchRegistryDenial::ExpiredAdmission => {
            SignalBranchRestoreDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalBranchRegistryDenial::UnknownBranch(_) => {
            SignalBranchRestoreDenial::UnknownBranch { branch_id }
        }
        SignalBranchRegistryDenial::RetirementInProgress(_) => {
            SignalBranchRestoreDenial::RetirementInProgress { branch_id }
        }
        SignalBranchRegistryDenial::TargetCellDenied(denial) => {
            restoration::map_restore_cell_denial(denial, branch_id)
        }
        SignalBranchRegistryDenial::OwnerMetadataOrdering => {
            SignalBranchRestoreDenial::OwnerCellMisuse { branch_id }
        }
        SignalBranchRegistryDenial::OwnerReentry => SignalBranchRestoreDenial::OwnerReentry,
        denial => SignalBranchRestoreDenial::OwnerDeniedNoMovement {
            error: registry_invariant_error("restore lookup", denial),
        },
    }
}

fn registry_invariant_error(operation: &str, denial: SignalBranchRegistryDenial) -> SignalError {
    SignalError::internal(format!(
        "Signal mutation port {operation} invariant failed: {denial:?}"
    ))
}
