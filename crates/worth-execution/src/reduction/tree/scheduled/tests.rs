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

#[test]
fn a_cancelled_subtree_keeps_its_completed_combines_and_charge() {
    use std::sync::atomic::AtomicUsize;
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    for serial in [false, true] {
        let cancellation = crate::CancellationSource::new();
        let mut policy = request(1, 2_000, 100);
        policy.cancellation = cancellation.token();
        let lease = if serial {
            crate::ExecutionResourceLease::serial(policy).unwrap()
        } else {
            authority().request_lease(policy).unwrap()
        };
        let calls = AtomicUsize::new(0);
        let mut observed = None;
        let outcome = run_scope(Some(&lease), 0, 0, |context| {
            let plan =
                ReductionPlan::try_from_sorted_unique(vec![PartitionIdentity::new(1)]).unwrap();
            let stopped = ReductionTree::build_scheduled_checked(
                &lease,
                BackendKind::Native,
                plan,
                vec![7_u64],
                0,
                |left: &u64, right: &u64| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    cancellation.cancel();
                    left + right
                },
                8,
                context,
            );
            let Err(ScheduledReductionError::Reduction(failure)) = stopped else {
                panic!("the next checked operation must observe cancellation");
            };
            assert!(matches!(
                failure.reason,
                ReductionRunStop::Hook(KernelStop::Cancelled)
            ));
            observed = Some(failure.metrics);
            Err::<(), _>(KernelFailure::<()>::Stop(KernelStop::Cancelled))
        });
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let metrics = observed.unwrap();
        assert_eq!(metrics.combine_calls, 1);
        // A singleton enters shape construction three times, evaluation twice,
        // and its first combine once; cancellation refuses the second combine.
        let work = 3 + 2 + 1;
        assert_eq!(metrics.structural_visits, 3 + 2);
        assert_eq!(metrics.recombined_nodes, 0);
        assert_eq!(metrics.charged_work, work);
        assert_eq!(metrics.charged_span, work);
        assert_eq!(outcome.report.charged_work(), work);
        assert_eq!(outcome.report.charged_span(), work);
        assert!(matches!(
            outcome.result,
            Err(ScopeStop::Failure(KernelFailure::Stop(
                KernelStop::Cancelled
            )))
        ));
    }
}
