//! A source read whose memory was refused ran out of recovery memory: the
//! same media reads under more of it, so that is recovery memory's limit.
//! Every other allocation refusal (the allocator's own failure, a capacity
//! past what was reserved, a length other than the one read) is the source
//! allocation's own failure.

use worth_store::physical_runtime::{
    PhysicalRecoveryObservationAllocationDenial as Observation,
    PhysicalRecoveryRejoinResidentAdmissionDenial as Admission,
    PhysicalRecoveryRejoinResidentDenial as Resident, RecoveryWalAllocationDenial as Wal,
    StoreRecoveryBindingSampleAllocationDenial as Sample,
    StoreRecoveryCheckpointBindingAllocationDenial as Binding,
};

use super::DiscoveryFailure;
use crate::entry::{
    PhysicalRecoveryBlockCause, PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDeclaration,
    PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure, PhysicalRecoverySourceDenial,
    PhysicalRecoverySourceReadAllocationDenial as Read,
};
use crate::orchestration::recovery_budget::RecoveryAllowance;

/// The recovery memory an allocation refusal says it needed, and what it was
/// handed of recovery's: `None` where memory was not what it refused.
pub(crate) trait MemoryRefused {
    fn memory_refused(&self) -> Option<(u64, u64)>;
}

/// Recovery memory's limit, where `cause` refused memory recovery declared:
/// the refusing owner's counts read beside the whole declared memory.
pub(crate) fn source_memory_limit(
    limits: &PhysicalRecoveryLimitDeclaration,
    cause: &impl MemoryRefused,
) -> Option<PhysicalRecoveryLimitFailure> {
    let (required, admitted) = cause.memory_refused()?;
    RecoveryAllowance::declared(limits, PhysicalRecoveryLimitDimension::RecoveryMemoryBytes)
        .beside(required, admitted)
        .map(Into::into)
}

/// The source read stopped: on recovery memory's limit where `cause`
/// refused memory, otherwise as the source allocation's failure. `denial`
/// names which read refused `cause`.
pub(super) fn source_allocation<C: MemoryRefused>(
    limits: &PhysicalRecoveryLimitDeclaration,
    cause: C,
    denial: impl FnOnce(C) -> PhysicalRecoverySourceDenial,
) -> DiscoveryFailure {
    let mut failure = stopped(limits, &cause);
    failure.source_denials.push(denial(cause));
    failure
}

/// [`source_allocation`], for a caller that names the read itself.
pub(super) fn stopped(
    limits: &PhysicalRecoveryLimitDeclaration,
    cause: &impl MemoryRefused,
) -> DiscoveryFailure {
    DiscoveryFailure::of(PhysicalRecoveryBlockCause::of(
        PhysicalRecoveryBlockKind::SourceAllocation,
        source_memory_limit(limits, cause),
    ))
}

impl MemoryRefused for Resident {
    fn memory_refused(&self) -> Option<(u64, u64)> {
        match self {
            Self::BudgetExceeded { required, admitted } => Some((*required, *admitted)),
            Self::MissingResidentAdmission
            | Self::OperationAllocation(_)
            | Self::SizeOverflow { .. }
            | Self::Allocation { .. }
            | Self::AllocatorExceededReservation { .. } => None,
        }
    }
}

impl MemoryRefused for Admission {
    fn memory_refused(&self) -> Option<(u64, u64)> {
        match self {
            Self::RecoveryMemoryBytes { observed, admitted } => Some((*observed, *admitted)),
            Self::MissingAllocation | Self::AlreadyAdmitted | Self::SizeOverflow => None,
        }
    }
}

impl MemoryRefused for Wal {
    fn memory_refused(&self) -> Option<(u64, u64)> {
        match self {
            Self::Ownership(admission) => admission.memory_refused(),
            Self::Backing { cause, .. } => cause.memory_refused(),
            Self::SizeOverflow | Self::AllocatorExceededReservation { .. } => None,
        }
    }
}

impl MemoryRefused for Observation {
    fn memory_refused(&self) -> Option<(u64, u64)> {
        match self {
            Self::Residency(cause)
            | Self::ListingResidency { cause, .. }
            | Self::PathResidency { cause, .. } => cause.memory_refused(),
            Self::StoreMismatch
            | Self::WalObservationUnavailable(_)
            | Self::UnqualifiedListingStorage
            | Self::UnqualifiedPathStorage
            | Self::AllocatorExceededReservation { .. } => None,
        }
    }
}

impl MemoryRefused for Binding {
    fn memory_refused(&self) -> Option<(u64, u64)> {
        match self {
            Self::Backing { cause, .. } => cause.memory_refused(),
            Self::StoreMismatch
            | Self::PoolMismatch
            | Self::BackingMismatch
            | Self::SizeOverflow
            | Self::AllocatorExceededReservation { .. } => None,
        }
    }
}

impl MemoryRefused for Sample {
    fn memory_refused(&self) -> Option<(u64, u64)> {
        match self {
            Self::LocalLimit { required, admitted } => Some((*required, *admitted)),
            Self::Ownership(cause) => cause.memory_refused(),
            Self::Backing { cause, .. } => cause.memory_refused(),
            Self::SizeOverflow
            | Self::BackingMismatch
            | Self::AllocatorExceededReservation { .. } => None,
        }
    }
}

impl MemoryRefused for Read {
    fn memory_refused(&self) -> Option<(u64, u64)> {
        match self {
            Self::Admission(cause) => cause.memory_refused(),
            Self::Residency(cause) => cause.memory_refused(),
            Self::Observation(cause) => cause.memory_refused(),
            Self::BindingDecode(cause) | Self::BindingBasis(cause) => cause.memory_refused(),
            Self::AllocatorExceededReservation { .. } | Self::ReadBufferLengthMismatch { .. } => {
                None
            }
        }
    }
}

#[cfg(test)]
#[path = "source_memory_tests.rs"]
mod tests;
