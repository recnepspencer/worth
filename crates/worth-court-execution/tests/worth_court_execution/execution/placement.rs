use core::{num::NonZeroUsize, time::Duration};
use std::sync::{Condvar, Mutex};

use worth_execution::{
    workers::ExecutionPlacement, CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionMap, LeaseDenial, LeaseRequest, MapKernelFailure, MapOutcome,
};
use worth_foundational::{DeterminismContract, PartitionIdentity};

fn request(placement: ExecutionPlacement) -> LeaseRequest {
    LeaseRequest {
        policy: placement.worker_policy(DeterminismContract::CanonicalBitwise),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

#[test]
fn admitted_partition_placement_bounds_workers_and_preserves_canonical_results() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig::for_workers(
        NonZeroUsize::new(4).unwrap(),
    ))
    .unwrap();
    let parallel = ExecutionPlacement::Parallel {
        max_workers: NonZeroUsize::new(2).unwrap(),
    };
    let oversized = ExecutionPlacement::Parallel {
        max_workers: NonZeroUsize::new(5).unwrap(),
    };
    assert!(matches!(
        authority.request_lease(request(oversized)),
        Err(LeaseDenial::WorkerLimitExceedsParent)
    ));

    let caller = std::thread::current().id();
    for placement in [ExecutionPlacement::Serial, parallel] {
        let lease = authority.request_lease(request(placement)).unwrap();
        assert_eq!(lease.placement(), placement);
        let map = ExecutionMap::<_, ()>::from_native_partitions(
            (1..=8).map(|id| (PartitionIdentity::new(id), id)).collect(),
        )
        .unwrap();
        let overlap = (Mutex::new(0), Condvar::new());
        let outcome = map.run_owned_native(Some(&lease), |input, _| {
            if placement == ExecutionPlacement::Serial {
                assert_eq!(std::thread::current().id(), caller);
            } else if input <= 2 {
                rendezvous(&overlap);
            }
            Ok::<_, MapKernelFailure<()>>(input)
        });
        let MapOutcome::Complete { values, report } = outcome else {
            panic!("the native map must complete");
        };
        assert!(values.into_iter().eq(1..=8));
        let workers = report.physical().active_workers_high_watermark();
        if placement == ExecutionPlacement::Serial {
            assert!(workers <= 1);
        } else {
            assert_eq!(workers, 2);
        }
    }
}

// Both callbacks remain active until two workers enter. A serial regression
// fails the court after a bounded wait.
fn rendezvous((arrivals, ready): &(Mutex<usize>, Condvar)) {
    let mut count = arrivals.lock().unwrap();
    *count += 1;
    ready.notify_all();
    let (count, _) = ready
        .wait_timeout_while(count, Duration::from_secs(5), |count| *count < 2)
        .unwrap();
    assert_eq!(*count, 2);
}
