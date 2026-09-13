use crate::branch::{
    ManagedSignalBranchReferenceAdmissionDenial, SignalBranchBasisObservationDenial,
    SignalBranchBasisReadmissionDenial, SignalBranchRetainedReadmissionDenial,
    SignalBranchRetentionAcquisitionDenial, SignalBranchRetentionReleaseDenial,
};
use crate::state::SignalBranchId;

use super::super::{
    SignalOwner, SignalOwnerAdmissionDenial, SignalOwnerOperationAdmission, SignalOwnerUnavailable,
};

pub(super) fn map_managed_observation_admission_denial(
    denial: ManagedSignalBranchReferenceAdmissionDenial,
) -> SignalBranchBasisObservationDenial {
    match denial {
        ManagedSignalBranchReferenceAdmissionDenial::OwnerUnavailable(unavailable) => {
            SignalBranchBasisObservationDenial::OwnerUnavailable(unavailable)
        }
        denial => SignalBranchBasisObservationDenial::ManagedReferenceDenied { denial },
    }
}

pub(in crate::branch::owner_services) fn map_observation_admission_denial(
    denial: SignalOwnerAdmissionDenial,
) -> SignalBranchBasisObservationDenial {
    match denial {
        SignalOwnerAdmissionDenial::ForeignOwner | SignalOwnerAdmissionDenial::OwnerUnavailable => {
            SignalBranchBasisObservationDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalOwnerAdmissionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        } => SignalBranchBasisObservationDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        },
        SignalOwnerAdmissionDenial::OwnerReentry => {
            SignalBranchBasisObservationDenial::OwnerReentry
        }
    }
}

pub(super) fn map_managed_readmission_admission_denial(
    denial: ManagedSignalBranchReferenceAdmissionDenial,
) -> SignalBranchBasisReadmissionDenial {
    match denial {
        ManagedSignalBranchReferenceAdmissionDenial::OwnerUnavailable(unavailable) => {
            SignalBranchBasisReadmissionDenial::OwnerUnavailable(unavailable)
        }
        denial => SignalBranchBasisReadmissionDenial::ManagedReferenceDenied { denial },
    }
}

pub(in crate::branch::owner_services) fn map_observation_retention_denial<D, I, T>(
    owner: &SignalOwner<D, I, T>,
    admission: &SignalOwnerOperationAdmission<'_>,
    denial: SignalBranchRetentionAcquisitionDenial,
    branch_id: SignalBranchId,
) -> SignalBranchBasisObservationDenial
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    match denial {
        SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(unavailable) => {
            SignalBranchBasisObservationDenial::OwnerUnavailable(unavailable)
        }
        SignalBranchRetentionAcquisitionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        } => SignalBranchBasisObservationDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        },
        SignalBranchRetentionAcquisitionDenial::OwnerReentry => {
            SignalBranchBasisObservationDenial::OwnerReentry
        }
        SignalBranchRetentionAcquisitionDenial::RetiredBranch { .. } => {
            owner.resolve_observation_retirement_denial(admission, branch_id)
        }
        denial => SignalBranchBasisObservationDenial::RetentionUnavailable { denial },
    }
}

pub(in crate::branch::owner_services) fn map_readmission_retention_denial<D, I, T>(
    owner: &SignalOwner<D, I, T>,
    admission: &SignalOwnerOperationAdmission<'_>,
    denial: SignalBranchRetentionAcquisitionDenial,
    branch_id: SignalBranchId,
) -> SignalBranchBasisReadmissionDenial
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    match denial {
        SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(unavailable) => {
            SignalBranchBasisReadmissionDenial::OwnerUnavailable(unavailable)
        }
        SignalBranchRetentionAcquisitionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        } => SignalBranchBasisReadmissionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        },
        SignalBranchRetentionAcquisitionDenial::OwnerReentry => {
            SignalBranchBasisReadmissionDenial::OwnerReentry
        }
        SignalBranchRetentionAcquisitionDenial::RetiredBranch { .. } => {
            owner.resolve_readmission_retirement_denial(admission, branch_id)
        }
        SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
            maximum_active_leases,
        } => SignalBranchBasisReadmissionDenial::UnavailableRetention {
            maximum_active_leases,
        },
        SignalBranchRetentionAcquisitionDenial::IdentityExhausted => {
            SignalBranchBasisReadmissionDenial::RetentionIdentityExhausted
        }
        _ => SignalBranchBasisReadmissionDenial::OwnerInvariantViolation { branch_id },
    }
}

pub(in crate::branch::owner_services) fn map_observation_readmission_denial<D, I, T>(
    owner: &SignalOwner<D, I, T>,
    admission: &SignalOwnerOperationAdmission<'_>,
    denial: SignalBranchBasisObservationDenial,
    branch_id: SignalBranchId,
) -> SignalBranchBasisReadmissionDenial
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    match denial {
        SignalBranchBasisObservationDenial::OwnerUnavailable(unavailable) => {
            SignalBranchBasisReadmissionDenial::OwnerUnavailable(unavailable)
        }
        SignalBranchBasisObservationDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        } => SignalBranchBasisReadmissionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        },
        SignalBranchBasisObservationDenial::OwnerReentry => {
            SignalBranchBasisReadmissionDenial::OwnerReentry
        }
        SignalBranchBasisObservationDenial::ManagedReferenceDenied { denial } => {
            SignalBranchBasisReadmissionDenial::ManagedReferenceDenied { denial }
        }
        SignalBranchBasisObservationDenial::UnknownBranch { branch_id } => {
            SignalBranchBasisReadmissionDenial::UnknownBranch { branch_id }
        }
        SignalBranchBasisObservationDenial::RetirementInProgress { branch_id } => {
            SignalBranchBasisReadmissionDenial::RetirementInProgress { branch_id }
        }
        SignalBranchBasisObservationDenial::RetiredBranch { branch_id } => {
            SignalBranchBasisReadmissionDenial::RetiredBranch { branch_id }
        }
        SignalBranchBasisObservationDenial::QuarantinedBranch { branch_id } => {
            SignalBranchBasisReadmissionDenial::QuarantinedBranch { branch_id }
        }
        SignalBranchBasisObservationDenial::OwnerCellMisuse { branch_id } => {
            SignalBranchBasisReadmissionDenial::OwnerCellMisuse { branch_id }
        }
        SignalBranchBasisObservationDenial::OwnerInvariantViolation { branch_id } => {
            SignalBranchBasisReadmissionDenial::OwnerInvariantViolation { branch_id }
        }
        SignalBranchBasisObservationDenial::InvalidOwnerObservation { .. } => {
            SignalBranchBasisReadmissionDenial::OwnerInvariantViolation { branch_id }
        }
        SignalBranchBasisObservationDenial::RetentionUnavailable { denial } => {
            map_readmission_retention_denial(owner, admission, denial, branch_id)
        }
    }
}

pub(super) fn map_retained_retention_denial(
    denial: SignalBranchRetentionAcquisitionDenial,
) -> SignalBranchRetainedReadmissionDenial {
    match denial {
        SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(unavailable) => {
            SignalBranchRetainedReadmissionDenial::OwnerUnavailable(unavailable)
        }
        SignalBranchRetentionAcquisitionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        } => SignalBranchRetainedReadmissionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        },
        SignalBranchRetentionAcquisitionDenial::OwnerReentry => {
            SignalBranchRetainedReadmissionDenial::OwnerReentry
        }
        SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
            maximum_active_leases,
        } => SignalBranchRetainedReadmissionDenial::UnavailableRetention {
            maximum_active_leases,
        },
        SignalBranchRetentionAcquisitionDenial::IdentityExhausted => {
            SignalBranchRetainedReadmissionDenial::RetentionIdentityExhausted
        }
        denial => SignalBranchRetainedReadmissionDenial::UnavailableExactTarget(denial),
    }
}

pub(super) fn map_retained_admission_denial(
    denial: SignalOwnerAdmissionDenial,
) -> SignalBranchRetainedReadmissionDenial {
    match denial {
        SignalOwnerAdmissionDenial::ForeignOwner | SignalOwnerAdmissionDenial::OwnerUnavailable => {
            SignalBranchRetainedReadmissionDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalOwnerAdmissionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        } => SignalBranchRetainedReadmissionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        },
        SignalOwnerAdmissionDenial::OwnerReentry => {
            SignalBranchRetainedReadmissionDenial::OwnerReentry
        }
    }
}

pub(in crate::branch::owner_services) fn map_basis_admission_denial(
    denial: SignalOwnerAdmissionDenial,
) -> SignalBranchBasisReadmissionDenial {
    match denial {
        SignalOwnerAdmissionDenial::ForeignOwner | SignalOwnerAdmissionDenial::OwnerUnavailable => {
            SignalBranchBasisReadmissionDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalOwnerAdmissionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        } => SignalBranchBasisReadmissionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        },
        SignalOwnerAdmissionDenial::OwnerReentry => {
            SignalBranchBasisReadmissionDenial::OwnerReentry
        }
    }
}

pub(super) fn map_retention_admission_denial(
    denial: SignalOwnerAdmissionDenial,
) -> SignalBranchRetentionAcquisitionDenial {
    match denial {
        SignalOwnerAdmissionDenial::ForeignOwner | SignalOwnerAdmissionDenial::OwnerUnavailable => {
            SignalBranchRetentionAcquisitionDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalOwnerAdmissionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        } => SignalBranchRetentionAcquisitionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        },
        SignalOwnerAdmissionDenial::OwnerReentry => {
            SignalBranchRetentionAcquisitionDenial::OwnerReentry
        }
    }
}

pub(super) fn map_release_admission_denial(
    denial: SignalOwnerAdmissionDenial,
) -> SignalBranchRetentionReleaseDenial {
    match denial {
        SignalOwnerAdmissionDenial::ForeignOwner | SignalOwnerAdmissionDenial::OwnerUnavailable => {
            SignalBranchRetentionReleaseDenial::OwnerUnavailable(SignalOwnerUnavailable)
        }
        SignalOwnerAdmissionDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        } => SignalBranchRetentionReleaseDenial::OperationCapacityExhausted {
            maximum_in_flight_operations,
        },
        SignalOwnerAdmissionDenial::OwnerReentry => {
            SignalBranchRetentionReleaseDenial::OwnerReentry
        }
    }
}
