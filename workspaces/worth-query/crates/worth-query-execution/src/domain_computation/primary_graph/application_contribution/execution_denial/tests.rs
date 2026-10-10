//! Each execution cause reaches Query as its own named cause. Before this
//! door, a counter overflow and a stopped nested pattern read as spent work,
//! an oversized result as spent retained bytes, every panic under a ceiling
//! as the reducer's, and a structural reduction refusal was `unreachable!`.

use std::num::NonZeroUsize;

use worth_execution::{
    CancellationToken, LeaseDenial, LeaseRequest, MapKernelFailure, MapKernelStop,
    MapMemoryOverflow, MapStop, MemoryLimitDenial, MemoryLimitLevel, ReductionDenial,
    ReductionMetrics, ReductionRunFailure, ReductionRunStop, SerialMemoryBudget, WorkCeilingDenial,
};
use worth_foundational::facade::PartitionIdentity;

use super::super::request_execution::{test_execution_authority, test_policy};
use super::*;

type Denial = WorthQueryPartitionedComputationDenial<u32>;
type Checkpoint = WorthQueryManagedComputationCheckpointDenial;
type Interruption = WorthQueryManagedComputationInterruption;

fn distinct<T: PartialEq + std::fmt::Debug>(values: &[T]) {
    for (at, left) in values.iter().enumerate() {
        for right in &values[at + 1..] {
            assert_ne!(left, right, "two causes share one name");
        }
    }
}

#[test]
fn every_lease_refusal_and_kernel_stop_is_its_own_cause() {
    let leases = [
        (LeaseDenial::WorkerLimitExceedsParent, Resource::WorkerLimit),
        (
            LeaseDenial::MemoryLimitExceedsParent,
            Resource::PolicyMemoryLimit,
        ),
        (LeaseDenial::WorkLimitExceedsParent, Resource::WorkLimit),
        (
            LeaseDenial::ChargedBytesOverflow,
            Resource::ChargedBytesOverflow,
        ),
        (
            LeaseDenial::UnrelatedNestedLease,
            Resource::NestedLeaseMisuse,
        ),
        (
            LeaseDenial::NoActiveExecutionScope,
            Resource::NoActiveExecutionScope,
        ),
        (
            LeaseDenial::EquivalenceContractUnavailable,
            Resource::EquivalenceContractUnavailable,
        ),
    ];
    for (denial, cause) in leases {
        assert_eq!(lease_denial(denial), cause);
    }
    distinct(&leases.map(|(_, cause)| cause));

    let stops = [
        (
            MapKernelStop::Cancelled,
            Checkpoint::Interrupted(Interruption::Cancelled),
        ),
        (
            MapKernelStop::DeadlineElapsed,
            Checkpoint::Interrupted(Interruption::DeadlineExceeded),
        ),
        (
            MapKernelStop::WorkCeiling,
            Checkpoint::Resource(Resource::WorkExhausted),
        ),
        (
            MapKernelStop::WorkCounterOverflow,
            Checkpoint::Resource(Resource::WorkCounterOverflow),
        ),
        (
            MapKernelStop::NestedStopped,
            Checkpoint::NestedPatternStopped,
        ),
    ];
    for (stop, cause) in stops {
        assert_eq!(checkpoint_denial(stop), cause);
    }
    distinct(&stops.map(|(_, cause)| cause));

    // One memory refusal has one spelling, from a reservation or a run, and
    // keeps the limit that refused it. Every lease of a request has its
    // policy, so a policy's depth is not a cause.
    let levels = [
        (
            MemoryLimitLevel::Policy { ancestor: 0 },
            WorthQueryMemoryLimitLevel::Policy,
        ),
        (
            MemoryLimitLevel::Policy { ancestor: 2 },
            WorthQueryMemoryLimitLevel::Policy,
        ),
        (
            MemoryLimitLevel::Process,
            WorthQueryMemoryLimitLevel::Process,
        ),
        (
            MemoryLimitLevel::Declared,
            WorthQueryMemoryLimitLevel::Declared,
        ),
    ];
    for (level, named) in levels {
        let refused = MemoryLimitDenial {
            requested: 9,
            admitted: 8,
            level,
        };
        let spelled = Resource::MemoryLimit {
            requested: 9,
            admitted: 8,
            level: named,
        };
        assert_eq!(memory_denial(refused), spelled);
        assert_eq!(lease_denial(LeaseDenial::MemoryExhausted(refused)), spelled);
    }
    distinct(&[
        WorthQueryMemoryLimitLevel::Policy,
        WorthQueryMemoryLimitLevel::Process,
        WorthQueryMemoryLimitLevel::Declared,
    ]);
    assert_eq!(
        Denial::from_map_overflow(MapMemoryOverflow),
        Denial::Resource(Resource::CapacityOverflow)
    );
}

/// A panic under a ceiling is named by the pattern the ceiling ran.
#[test]
fn a_panic_under_a_ceiling_is_the_running_patterns() {
    let partition = PartitionIdentity::new(3);
    let kernel = || Denial::Partition {
        partition,
        cause: WorthQueryComputationPartitionStop::Panicked,
    };
    assert_eq!(
        Denial::from_work_ceiling(WorkCeilingDenial::Panicked, kernel()),
        kernel()
    );
    assert_eq!(
        Denial::from_work_ceiling(WorkCeilingDenial::Panicked, Denial::ReducerPanicked),
        Denial::ReducerPanicked
    );
}

#[test]
fn every_reduction_stop_is_its_own_cause() {
    let reduced = |reason| {
        Denial::from_reduction(ReductionRunFailure {
            reason,
            metrics: ReductionMetrics::default(),
        })
    };
    let partition = PartitionIdentity::new(1);
    assert_eq!(
        reduced(ReductionRunStop::WorkCounterOverflow),
        Denial::Resource(Resource::WorkCounterOverflow)
    );
    assert_eq!(
        reduced(ReductionRunStop::ResultCapacityExceeded),
        Denial::Resource(Resource::ResultCapacityExceeded)
    );
    assert_eq!(
        reduced(ReductionRunStop::Hook(MapKernelStop::NestedStopped)),
        Denial::NestedPatternStopped
    );
    let structural = [
        (
            ReductionDenial::IdentitiesNotCanonical,
            ReductionInput::IdentitiesNotCanonical,
        ),
        (
            ReductionDenial::ValueCountMismatch,
            ReductionInput::ValueCountMismatch,
        ),
        (
            ReductionDenial::CoverageMismatch,
            ReductionInput::CoverageMismatch,
        ),
        (
            ReductionDenial::UnknownIdentity(partition),
            ReductionInput::UnknownPartition { partition },
        ),
        (
            ReductionDenial::IdentityAlreadyPresent(partition),
            ReductionInput::PartitionAlreadyPresent { partition },
        ),
    ];
    for (denial, cause) in structural {
        assert_eq!(
            reduced(ReductionRunStop::Denial(denial)),
            Denial::ReductionInputInvalid(cause)
        );
    }
    distinct(&structural.map(|(_, cause)| cause));
}

/// Each refusal a partition's kernel makes comes back as that partition's
/// own cause, through the kernel and the map unchanged.
#[test]
fn a_partitions_refusal_round_trips_through_its_kernel() {
    let partition = PartitionIdentity::new(5);
    let through = |denial: WorthQueryManagedComputationDenial<u32>| {
        Denial::from_map_stop(MapStop::Failure {
            identity: partition,
            cause: kernel_failure(denial),
        })
    };
    let resources = [
        Resource::WorkExhausted,
        Resource::WorkCounterOverflow,
        Resource::RetainedBytesExhausted,
        Resource::ResultCapacityExceeded,
        Resource::CapacityOverflow,
        Resource::MemoryLimit {
            requested: 2,
            admitted: 1,
            level: WorthQueryMemoryLimitLevel::Process,
        },
        Resource::WorkerLimit,
        Resource::PolicyMemoryLimit,
        Resource::WorkLimit,
        Resource::NestedLeaseMisuse,
        Resource::NoActiveExecutionScope,
        Resource::EquivalenceContractUnavailable,
    ];
    distinct(&resources);
    for resource in resources {
        assert_eq!(
            through(WorthQueryManagedComputationDenial::Resource(resource)),
            Denial::Partition {
                partition,
                cause: WorthQueryComputationPartitionStop::Resource(resource),
            }
        );
    }
    assert_eq!(
        through(WorthQueryManagedComputationDenial::Owner(7)),
        Denial::Partition {
            partition,
            cause: WorthQueryComputationPartitionStop::Owner(7),
        }
    );
    assert_eq!(
        through(WorthQueryManagedComputationDenial::Interrupted(
            Interruption::DeadlineExceeded
        )),
        Denial::Partition {
            partition,
            cause: WorthQueryComputationPartitionStop::Interrupted(Interruption::DeadlineExceeded),
        }
    );
    assert_eq!(
        through(WorthQueryManagedComputationDenial::NestedPatternStopped),
        Denial::Partition {
            partition,
            cause: WorthQueryComputationPartitionStop::NestedPatternStopped,
        }
    );
    assert_eq!(
        Denial::from_map_stop(MapStop::Failure {
            identity: partition,
            cause: MapKernelFailure::ResultCapacityExceeded,
        }),
        Denial::Partition {
            partition,
            cause: WorthQueryComputationPartitionStop::Resource(Resource::ResultCapacityExceeded),
        }
    );
}

/// A request's runs dispatch on children of its lease that copy its policy.
/// A child refused by the room its parent left and the same request run
/// serially are refused by one policy, and say so the same way.
#[test]
fn a_dispatch_child_and_a_serial_run_are_refused_by_the_same_policy() {
    let policy = test_policy(NonZeroUsize::new(2).unwrap(), 1000);
    let request = |policy| LeaseRequest {
        policy,
        deadline: None,
        cancellation: CancellationToken::new(),
    };
    let parent = test_execution_authority()
        .request_lease(request(policy))
        .unwrap();
    let _routing = parent.reserve_memory(600).unwrap();
    let child = parent.child(request(policy)).unwrap();
    let leased = child.reserve_memory(500).unwrap_err();
    assert_eq!(leased.level, MemoryLimitLevel::Policy { ancestor: 1 });

    let serial = SerialMemoryBudget::from_policy(&policy);
    let _routing = serial.reserve(600).unwrap();
    let serially = serial.reserve(500).unwrap_err();
    assert_eq!(serially.level, MemoryLimitLevel::Policy { ancestor: 0 });

    let refused = Resource::MemoryLimit {
        requested: 500,
        admitted: 400,
        level: WorthQueryMemoryLimitLevel::Policy,
    };
    assert_eq!(memory_denial(leased), refused);
    assert_eq!(memory_denial(serially), refused);
}
