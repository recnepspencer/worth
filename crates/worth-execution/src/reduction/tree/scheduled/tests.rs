//! A discarded frontier task still contributes its request-owned stop.
use super::*;
use crate::backend::{run_checked_batch, run_scope, ScopeStop};
use crate::tests::{authority, batch, request, TEST_LOCK};
use std::sync::atomic::{AtomicBool, Ordering};
use worth_foundational::PartitionIdentity;

#[test]
fn a_frontier_suffix_request_dominates_the_first_panic() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = authority().request_lease(request(2, 2_000, 100)).unwrap();
    let unrelated = authority().request_lease(request(1, 500, 100)).unwrap();
    let saw_panic = AtomicBool::new(false);
    let saw_refusal = AtomicBool::new(false);
    let outer = run_scope(None, 0, 0, |parent| {
        let inner = run_scope(None, 0, 0, |context| {
            // SplitMix priorities put 3 at the root, leaving independent
            // frontier leaves 1 and 4 in canonical order.
            let plan = ReductionPlan::try_from_sorted_unique(vec![
                PartitionIdentity::new(1),
                PartitionIdentity::new(3),
                PartitionIdentity::new(4),
            ])
            .unwrap();
            let result = ReductionTree::build_scheduled_checked(
                &lease,
                BackendKind::Serial,
                plan,
                vec![1_u64, 3, 4],
                0,
                |left: &u64, right: &u64| {
                    if *right == 1 {
                        saw_panic.store(true, Ordering::SeqCst);
                        panic!("first frontier task");
                    }
                    if *right == 4 {
                        let refused = run_checked_batch(
                            Some(&unrelated),
                            &batch(&[1], 0),
                            BackendKind::Serial,
                            &|_, _| Ok::<_, KernelFailure<()>>(1_u64),
                        );
                        assert_eq!(
                            refused.stop,
                            Some(BatchStop::Admission(LeaseDenial::UnrelatedNestedLease))
                        );
                        saw_refusal.store(true, Ordering::SeqCst);
                    }
                    left + right
                },
                8,
                context,
            );
            assert!(saw_panic.load(Ordering::SeqCst));
            assert!(saw_refusal.load(Ordering::SeqCst));
            assert!(matches!(result, Err(ScheduledReductionError::Reduction(_))));
            result.map(|_| ()).map_err(|_| KernelFailure::Domain(()))
        });
        assert!(inner.result.is_err());
        assert_eq!(parent.take_nested_computation_stop(), None);
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(matches!(
        outer.result,
        Err(ScopeStop::Failure(KernelFailure::Stop(
            KernelStop::NestedStopped
        )))
    ));
}
