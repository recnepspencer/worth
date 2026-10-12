//! A later stopped task cannot discard an earlier canonical join.
use super::*;
use worth_execution::{
    CancellationSource, ExecutionResourceLease, ExecutionWorkCeiling, MapKernelStop,
    SerialMemoryBudget, SerialRequest,
};

#[test]
fn a_wide_cancelled_frontier_settles_every_earlier_canonical_join() {
    let _guard = test_serialization::guard(&TEST_LOCK);
    let mut oracle = None;
    // The fixed shape of 1..=9 has four frontier leaves: 2, 4, 6, 9.
    // Node 1 finishes between leaves 2 and 4. Stop inside leaf 4.
    for backing in [0, 1, 2, 3, 4, 5] {
        let cancellation = CancellationSource::new();
        let workers = match backing {
            0 | 1 | 2 => 1,
            3 => 2,
            4 => 3,
            5 => 4,
            _ => unreachable!(),
        };
        let policy = LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                if backing < 2 {
                    ExecutionPosture::Serial
                } else {
                    ExecutionPosture::Automatic
                },
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(workers).unwrap(), 8 << 20, 10_000),
            ),
            deadline: None,
            cancellation: cancellation.token(),
        };
        let lease = match backing {
            0 => None,
            1 => Some(ExecutionResourceLease::serial(policy).unwrap()),
            _ => Some(authority().request_lease(policy).unwrap()),
        };
        let serial = SerialRequest::from_memory(
            SerialMemoryBudget::new(8 << 20),
            cancellation.token(),
            None,
        );
        let input = map(9);
        let first = (Mutex::new(false), std::sync::Condvar::new());
        let run = || {
            input.run_reduce(
                lease.as_ref(),
                mapped_u64,
                0,
                |left: &u64, right: &u64| {
                    if (*left, *right) == if workers < 3 { (1, 2) } else { (2, 0) } {
                        *first.0.lock().unwrap() = true;
                        first.1.notify_all();
                    }
                    if *left == 0 && *right == 4 {
                        // Failure watchdog only; a passing schedule is released by
                        // the first canonical task's completion, never by time.
                        let (done, guard) = first
                            .1
                            .wait_timeout_while(
                                first.0.lock().unwrap(),
                                std::time::Duration::from_secs(60),
                                |done| !*done,
                            )
                            .unwrap();
                        assert!(
                            *done && !guard.timed_out(),
                            "first canonical task never completed"
                        );
                        cancellation.cancel();
                    }
                    left + right
                },
                8,
                0,
            )
        };
        let stopped = if lease.is_none() {
            ExecutionWorkCeiling::new(10_000)
                .run_serial(&serial, run)
                .unwrap()
                .0
        } else {
            run()
        };
        let Err(ReduceInputDenial::ReductionStopped { failure, report }) = stopped else {
            panic!("the selected leaf must stop during its second combine");
        };
        assert!(matches!(
            failure.reason,
            ReductionRunStop::Hook(MapKernelStop::Cancelled)
        ));
        assert_eq!(
            failure.metrics.combine_calls,
            2 + 2 + 1,
            "leaf 2, parent 1, then the first combine of leaf 4"
        );
        let observed = (
            failure.metrics,
            report.charged_work(),
            report.charged_span(),
        );
        if let Some(expected) = &oracle {
            assert_eq!(&observed, expected, "backing {backing}");
        } else {
            oracle = Some(observed);
        }
    }
}
