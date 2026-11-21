//! Total preservation of execution stops at the derived-view boundary.

use super::WorthQueryManagedDerivedViewDenial as Denial;
use worth_execution::{
    LeaseDenial, MapKernelFailure, MapKernelStop, MapStop, MemoryLimitLevel, WorkCeilingDenial,
};
use worth_relational::facade::identity::EntityId;

pub fn lease(denial: LeaseDenial) -> Denial {
    match denial {
        LeaseDenial::WorkerLimitExceedsParent => Denial::WorkerLimitExceedsParent,
        LeaseDenial::MemoryLimitExceedsParent => Denial::MemoryLimitExceedsParent,
        LeaseDenial::WorkLimitExceedsParent => Denial::WorkLimitExceedsParent,
        LeaseDenial::MemoryExhausted(memory) => match memory {
            worth_execution::MemoryLimitDenial::CounterOverflow { .. } => {
                Denial::ChargedBytesOverflow
            }
            worth_execution::MemoryLimitDenial::Capacity { level, .. } => match level {
                MemoryLimitLevel::Policy { .. } => Denial::PolicyMemoryExhausted(memory),
                MemoryLimitLevel::Process => Denial::ProcessMemoryExhausted(memory),
                MemoryLimitLevel::Declared => Denial::DeclaredMemoryExhausted(memory),
            },
        },
        LeaseDenial::ChargedBytesOverflow => Denial::ChargedBytesOverflow,
        LeaseDenial::UnrelatedNestedLease => Denial::UnrelatedNestedLease,
        LeaseDenial::NoActiveExecutionScope => Denial::NoActiveExecutionScope,
        LeaseDenial::EquivalenceContractUnavailable => Denial::EquivalenceContractUnavailable,
    }
}

pub fn kernel(stop: MapKernelStop, root: Option<EntityId>) -> Denial {
    match stop {
        MapKernelStop::Cancelled => Denial::Cancelled,
        MapKernelStop::DeadlineElapsed => Denial::DeadlineElapsed,
        MapKernelStop::WorkCounterOverflow => Denial::WorkCounterOverflow,
        MapKernelStop::AccountingUnavailable => Denial::AccountingUnavailable,
        MapKernelStop::WorkCeiling => Denial::WorkExhausted { root },
        MapKernelStop::NestedStopped => Denial::NestedStopped,
    }
}

pub fn scope(stop: WorkCeilingDenial) -> Denial {
    match stop {
        WorkCeilingDenial::Admission(cause) => lease(cause),
        WorkCeilingDenial::Stopped(cause) => kernel(cause, None),
        WorkCeilingDenial::Panicked => Denial::OwnerPanic,
    }
}

pub fn map(stop: MapStop<Denial>, roots: &[EntityId]) -> Denial {
    let root_at =
        |identity: worth_foundational::PartitionIdentity| roots[(identity.value() - 1) as usize];
    match stop {
        MapStop::Admission(cause) => lease(cause),
        MapStop::Allocation(_) => Denial::AllocationUnavailable,
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

pub fn rounds(cause: worth_execution::RoundsDenial) -> Denial {
    match cause {
        worth_execution::RoundsDenial::RoundCountOverflow => Denial::CapacityOverflow,
        worth_execution::RoundsDenial::MemoryUnavailable => Denial::AllocationUnavailable,
    }
}

pub fn owner(cause: MapKernelFailure<Denial>) -> Denial {
    match cause {
        MapKernelFailure::Domain(cause) => cause,
        MapKernelFailure::Stop(cause) => kernel(cause, None),
        MapKernelFailure::Panic => Denial::OwnerPanic,
        MapKernelFailure::ResultCapacityExceeded => Denial::OwnerResultCapacityExceeded,
    }
}

pub fn session(
    cause: crate::domain_computation::provider_session::WorthQueryManagedGraphReadDenial,
    root: EntityId,
    plan: &crate::domain_computation::primary_graph::application_query::read_execution::read_plan::ReadPlan<'_>,
) -> Denial {
    use crate::domain_computation::primary_graph::application_query::{
        authorized_read::WorthQueryAuthorizedApplicationReadDenial as Read,
        one_shot::map_authorized_read_denial, read_execution::managed_read_denial,
    };
    match managed_read_denial(cause, plan) {
        Err(stop) => kernel(stop, None),
        Ok(Read::Session) => Denial::IncompleteDependencies,
        Ok(
            read @ (Read::Read(_)
            | Read::Authorization(_)
            | Read::StalePrincipal
            | Read::ScopeFreshness(_)),
        ) => Denial::ReadDenied {
            root,
            denial: map_authorized_read_denial(read, plan.name),
        },
    }
}
