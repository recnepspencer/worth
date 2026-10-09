//! Every execution cause remains distinct at the application boundary.
use super::{super::Denial, denial::*};
use worth_execution::{
    LeaseDenial, MapKernelFailure, MapKernelStop, MapStop, MemoryLimitDenial, MemoryLimitLevel,
    WorkCeilingDenial,
};
use worth_foundational::PartitionIdentity;
use worth_relational::facade::identity::{EntityId, PartitionId};

#[test]
fn reconstruction_denial_conversion_is_total_and_preserves_payloads() {
    for (source, expected) in [
        (
            LeaseDenial::WorkerLimitExceedsParent,
            Denial::WorkerLimitExceedsParent,
        ),
        (
            LeaseDenial::MemoryLimitExceedsParent,
            Denial::MemoryLimitExceedsParent,
        ),
        (
            LeaseDenial::WorkLimitExceedsParent,
            Denial::WorkLimitExceedsParent,
        ),
        (
            LeaseDenial::ChargedBytesOverflow,
            Denial::ChargedBytesOverflow,
        ),
        (
            LeaseDenial::UnrelatedNestedLease,
            Denial::UnrelatedNestedLease,
        ),
        (
            LeaseDenial::NoActiveExecutionScope,
            Denial::NoActiveExecutionScope,
        ),
        (
            LeaseDenial::EquivalenceContractUnavailable,
            Denial::EquivalenceContractUnavailable,
        ),
    ] {
        assert_eq!(lease(source), expected);
        assert_eq!(scope(WorkCeilingDenial::Admission(source)), expected);
    }
    for level in [
        MemoryLimitLevel::Policy { ancestor: 0 },
        MemoryLimitLevel::Policy { ancestor: 3 },
        MemoryLimitLevel::Process,
        MemoryLimitLevel::Declared,
    ] {
        let memory = MemoryLimitDenial {
            requested: 123,
            admitted: 45,
            level,
        };
        let expected = match level {
            MemoryLimitLevel::Policy { .. } => Denial::PolicyMemoryExhausted(memory),
            MemoryLimitLevel::Process => Denial::ProcessMemoryExhausted(memory),
            MemoryLimitLevel::Declared => Denial::DeclaredMemoryExhausted(memory),
        };
        assert_eq!(lease(LeaseDenial::MemoryExhausted(memory)), expected);
    }
    let root = EntityId::new(PartitionId::main(), 7, 1);
    for (source, expected) in [
        (MapKernelStop::Cancelled, Denial::Cancelled),
        (MapKernelStop::DeadlineElapsed, Denial::DeadlineElapsed),
        (
            MapKernelStop::WorkCounterOverflow,
            Denial::WorkCounterOverflow,
        ),
        (
            MapKernelStop::WorkCeiling,
            Denial::WorkExhausted { root: Some(root) },
        ),
        (MapKernelStop::NestedStopped, Denial::NestedStopped),
    ] {
        assert_eq!(kernel(source, Some(root)), expected);
        assert_eq!(
            map(
                MapStop::Failure {
                    identity: PartitionIdentity::new(1),
                    cause: MapKernelFailure::Stop(source)
                },
                &[root]
            ),
            expected
        );
    }
    for (cause, expected) in [
        (
            MapKernelFailure::Domain(Denial::StaleSource),
            Denial::StaleSource,
        ),
        (MapKernelFailure::Panic, Denial::KernelPanic { root }),
        (
            MapKernelFailure::ResultCapacityExceeded,
            Denial::ResultCapacityExceeded { root },
        ),
    ] {
        assert_eq!(
            map(
                MapStop::Failure {
                    identity: PartitionIdentity::new(1),
                    cause
                },
                &[root]
            ),
            expected
        );
    }
    assert_eq!(
        map(
            MapStop::WorkExhausted {
                identity: PartitionIdentity::new(1)
            },
            &[root]
        ),
        Denial::WorkExhausted { root: Some(root) }
    );
    assert_eq!(scope(WorkCeilingDenial::Panicked), Denial::OwnerPanic);
}

#[test]
fn owner_declaration_and_result_refusals_keep_distinct_causes() {
    assert_eq!(
        super::denial::rounds(worth_execution::RoundsDenial::RoundCountOverflow),
        Denial::CapacityOverflow
    );
    assert_eq!(
        super::denial::rounds(worth_execution::RoundsDenial::MemoryUnavailable),
        Denial::AllocationUnavailable
    );
    assert_eq!(
        super::denial::owner(MapKernelFailure::ResultCapacityExceeded),
        Denial::OwnerResultCapacityExceeded
    );
    assert_eq!(
        super::denial::owner(MapKernelFailure::Panic),
        Denial::OwnerPanic
    );
}
