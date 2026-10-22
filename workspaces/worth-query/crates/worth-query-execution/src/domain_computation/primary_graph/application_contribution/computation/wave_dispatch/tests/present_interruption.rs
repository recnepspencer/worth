//! Dispatch-time cancellation and deadline have the same facts at every width.
use super::*;

#[test]
fn a_present_interruption_dispatches_no_member_at_any_matrix_width() {
    for deadline in [false, true] {
        for width in [None, Some(1), Some(2), Some(5)] {
            let cancellation = WorthQueryCancellationSource::new();
            if !deadline {
                cancellation.cancel();
            }
            let scope =
                WorthQueryRequestScope::new(deadline.then(Instant::now), cancellation.token());
            let policy = worker_policy(width.unwrap_or(1));
            let placement = width.map_or(RuntimeWorldExecutionPlacement::Serial(policy), |_| {
                RuntimeWorldExecutionPlacement::Leased {
                    authority: test_authority(),
                    policy,
                }
            });
            let root = QueryRequestExecution::open(placement, &scope);
            let entered = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let members = (0..4)
                .map(|_| {
                    let entered = entered.clone();
                    PreparedMember {
                        requests: vec![PreparedCompute::new(move |_| {
                            entered.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        })],
                    }
                })
                .collect();
            let request = root.execution_request().unwrap();
            let outcome = request.run_native(|_| dispatch(&root, members));
            let expected = worth_execution::WorkCeilingDenial::Stopped(if deadline {
                worth_execution::MapKernelStop::DeadlineElapsed
            } else {
                worth_execution::MapKernelStop::Cancelled
            });
            match outcome {
                Err(actual) => assert_eq!(
                    actual, expected,
                    "present interruption keeps its typed kind before dispatch"
                ),
                Ok(_) => panic!("present interruption must stop before dispatch"),
            }
            assert_eq!(
                entered.load(std::sync::atomic::Ordering::SeqCst),
                0,
                "present interruption dispatches and charges no compute"
            );
        }
    }
}
