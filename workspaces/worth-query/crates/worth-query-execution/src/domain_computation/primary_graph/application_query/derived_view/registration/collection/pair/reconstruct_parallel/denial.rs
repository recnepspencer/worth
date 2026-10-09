//! Total preservation of execution stops at the derived-view boundary.

use super::super::Denial;
use worth_execution::{
    ChargedBytes, LeaseDenial, MapKernelFailure, MapKernelStop, MapStop, MemoryLimitLevel,
    WorkCeilingDenial,
};
use worth_relational::facade::identity::EntityId;

pub(super) fn lease(denial: LeaseDenial) -> Denial {
    match denial {
        LeaseDenial::WorkerLimitExceedsParent => Denial::WorkerLimitExceedsParent,
        LeaseDenial::MemoryLimitExceedsParent => Denial::MemoryLimitExceedsParent,
        LeaseDenial::WorkLimitExceedsParent => Denial::WorkLimitExceedsParent,
        LeaseDenial::MemoryExhausted(memory) => match memory.level {
            MemoryLimitLevel::Policy { ancestor: _ } => Denial::PolicyMemoryExhausted(memory),
            MemoryLimitLevel::Process => Denial::ProcessMemoryExhausted(memory),
            MemoryLimitLevel::Declared => Denial::DeclaredMemoryExhausted(memory),
        },
        LeaseDenial::ChargedBytesOverflow => Denial::ChargedBytesOverflow,
        LeaseDenial::UnrelatedNestedLease => Denial::UnrelatedNestedLease,
        LeaseDenial::NoActiveExecutionScope => Denial::NoActiveExecutionScope,
        LeaseDenial::EquivalenceContractUnavailable => Denial::EquivalenceContractUnavailable,
    }
}

pub(super) fn kernel(stop: MapKernelStop, root: Option<EntityId>) -> Denial {
    match stop {
        MapKernelStop::Cancelled => Denial::Cancelled,
        MapKernelStop::DeadlineElapsed => Denial::DeadlineElapsed,
        MapKernelStop::WorkCounterOverflow => Denial::WorkCounterOverflow,
        MapKernelStop::WorkCeiling => Denial::WorkExhausted { root },
        MapKernelStop::NestedStopped => Denial::NestedStopped,
    }
}

pub(super) fn scope(stop: WorkCeilingDenial) -> Denial {
    match stop {
        WorkCeilingDenial::Admission(cause) => lease(cause),
        WorkCeilingDenial::Stopped(cause) => kernel(cause, None),
        WorkCeilingDenial::Panicked => Denial::OwnerPanic,
    }
}

pub(super) fn map(stop: MapStop<Denial>, roots: &[EntityId]) -> Denial {
    let root_at =
        |identity: worth_foundational::PartitionIdentity| roots[(identity.value() - 1) as usize];
    match stop {
        MapStop::Admission(cause) => lease(cause),
        MapStop::WorkExhausted { identity } => Denial::WorkExhausted {
            root: Some(root_at(identity)),
        },
        MapStop::Failure { identity, cause } => match cause {
            MapKernelFailure::Stop(stop) => kernel(stop, Some(root_at(identity))),
            MapKernelFailure::Domain(denial) => denial,
            MapKernelFailure::Panic => Denial::KernelPanic {
                root: root_at(identity),
            },
            MapKernelFailure::ResultCapacityExceeded => Denial::ResultCapacityExceeded {
                root: root_at(identity),
            },
        },
    }
}

pub(super) fn rounds(cause: worth_execution::RoundsDenial) -> Denial {
    match cause {
        worth_execution::RoundsDenial::RoundCountOverflow => Denial::CapacityOverflow,
        worth_execution::RoundsDenial::MemoryUnavailable => Denial::AllocationUnavailable,
    }
}

pub(super) fn owner(cause: MapKernelFailure<Denial>) -> Denial {
    match cause {
        MapKernelFailure::Domain(cause) => cause,
        MapKernelFailure::Stop(cause) => kernel(cause, None),
        MapKernelFailure::Panic => Denial::OwnerPanic,
        MapKernelFailure::ResultCapacityExceeded => Denial::OwnerResultCapacityExceeded,
    }
}

impl ChargedBytes for Denial {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::ReadDenied {
                root: _root,
                denial,
            } => denial.additional_charged_bytes(),
            Self::PolicyMemoryExhausted(worth_execution::MemoryLimitDenial {
                requested: _requested,
                admitted: _admitted,
                level: _level,
            })
            | Self::ProcessMemoryExhausted(worth_execution::MemoryLimitDenial {
                requested: _requested,
                admitted: _admitted,
                level: _level,
            })
            | Self::DeclaredMemoryExhausted(worth_execution::MemoryLimitDenial {
                requested: _requested,
                admitted: _admitted,
                level: _level,
            }) => 0,
            Self::WorkExhausted { root: _root } => 0,
            Self::DuplicateRoot { root: _root }
            | Self::KernelPanic { root: _root }
            | Self::ResultCapacityExceeded { root: _root } => 0,
            Self::BatchResource { root: _, denial: _ }
            | Self::InvalidLimits
            | Self::ViewCapacityExceeded
            | Self::EntryCapacityExceeded
            | Self::RetainedBytesExceeded
            | Self::ForeignApplication
            | Self::ForeignInstallation
            | Self::ForeignQuery
            | Self::AuthorizationRequired
            | Self::ForeignBranch
            | Self::StaleSource
            | Self::IncompleteDependencies
            | Self::ColdReconstructionRequired
            | Self::MembershipReconciliationRequired
            | Self::EntryRefreshRequired
            | Self::QueryExecutionDenied
            | Self::ViewRevisionExhausted
            | Self::Disposed
            | Self::WorkerLimitExceedsParent
            | Self::MemoryLimitExceedsParent
            | Self::WorkLimitExceedsParent
            | Self::ChargedBytesOverflow
            | Self::CapacityOverflow
            | Self::AllocationUnavailable
            | Self::OwnerResultCapacityExceeded
            | Self::UnrelatedNestedLease
            | Self::NoActiveExecutionScope
            | Self::EquivalenceContractUnavailable
            | Self::Cancelled
            | Self::DeadlineElapsed
            | Self::WorkCounterOverflow
            | Self::NestedStopped
            | Self::OwnerPanic => 0,
        }
    }
}

pub(super) fn session(
    cause: crate::domain_computation::provider_session::WorthQueryManagedGraphReadDenial,
) -> Denial {
    use crate::domain_computation::provider_session::WorthQueryManagedGraphReadDenial as Cause;
    match cause {
        Cause::MutationSession
        | Cause::ForeignBasis
        | Cause::ForeignGraph
        | Cause::ForeignReadProof
        | Cause::TerminalReleaseMismatch => Denial::IncompleteDependencies,
    }
}
