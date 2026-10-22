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
        assert_eq!(metrics.accounted_metrics().unwrap().combine_calls, 1);
        // A singleton enters shape construction three times, evaluation twice,
        // and its first combine once; cancellation refuses the second combine.
        let work = 3 + 2 + 1;
        assert_eq!(
            metrics.accounted_metrics().unwrap().structural_visits,
            3 + 2
        );
        assert_eq!(metrics.accounted_metrics().unwrap().recombined_nodes, 0);
        assert_eq!(metrics.accounted_metrics().unwrap().charged_work, work);
        assert_eq!(metrics.accounted_metrics().unwrap().charged_span, work);
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

#[test]
fn a_cancelled_canonical_prefix_agrees_at_every_frontier_width() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let physical = cancellation_authority();
    let mut oracle = None;
    // Canonical task order constructs the interruption; frontier widths still
    // select different Node/Task event streams. Scheduling is not the oracle.
    for backing in [0, 1, 2, 3, 4, 5] {
        let cancellation = crate::CancellationSource::new();
        let mut policy = cancellation_request(if backing >= 3 { backing - 1 } else { 1 });
        policy.cancellation = cancellation.token();
        let lease = match backing {
            0 => None,
            1 => Some(crate::ExecutionResourceLease::serial(policy).unwrap()),
            2..=5 => Some(physical.request_lease(policy).unwrap()),
            _ => unreachable!(),
        };
        let mut measured = None;
        let outcome = run_scope(lease.as_ref(), 0, 0, |context| {
            let plan = ReductionPlan::try_from_sorted_unique(
                [1, 3, 4].map(PartitionIdentity::new).to_vec(),
            )
            .unwrap();
            let combine = |left: &u64, right: &u64| {
                cancellation.cancel();
                left + right
            };
            let result = if let Some(lease) = &lease {
                ReductionTree::build_scheduled_checked(
                    lease,
                    BackendKind::Serial,
                    plan,
                    vec![1_u64, 3, 4],
                    0,
                    combine,
                    8,
                    context,
                )
                .map_err(|error| match error {
                    ScheduledReductionError::Reduction(failure) => failure,
                    ScheduledReductionError::Admission(denial) => panic!("admission: {denial:?}"),
                })
            } else {
                ReductionTree::from_plan_checked(plan, vec![1_u64, 3, 4], 0, combine, 8, || {
                    if cancellation.token().is_cancelled() {
                        Err(KernelStop::Cancelled)
                    } else {
                        context.checkpoint(1)
                    }
                })
            };
            let failure = result
                .err()
                .expect("the second combine observes cancellation");
            assert_eq!(
                failure.metrics.accounted_metrics().unwrap().combine_calls,
                1
            );
            assert!(matches!(
                failure.reason,
                ReductionRunStop::Hook(KernelStop::Cancelled)
            ));
            measured = Some(failure.metrics);
            Err::<(), _>(KernelFailure::<()>::Stop(KernelStop::Cancelled))
        });
        let metrics = measured.unwrap();
        assert_eq!(
            outcome.report.charged_work(),
            metrics.accounted_metrics().unwrap().charged_work
        );
        assert_eq!(
            outcome.report.charged_span(),
            metrics.accounted_metrics().unwrap().charged_span
        );
        let observed = (
            metrics,
            outcome.report.charged_work(),
            outcome.report.charged_span(),
            matches!(
                outcome.result,
                Err(ScopeStop::Failure(KernelFailure::Stop(
                    KernelStop::Cancelled
                )))
            ),
        );
        if let Some(expected) = &oracle {
            assert_eq!(&observed, expected, "backing {backing}");
        } else {
            oracle = Some(observed);
        }
    }
}

#[test]
fn a_stop_after_the_last_combine_waits_for_the_next_checkpoint_at_every_width() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let physical = cancellation_authority();
    let mut oracle = None;
    // Canonical task order constructs the interruption; frontier widths still
    // select different Node/Task event streams. Scheduling is not the oracle.
    for backing in [0, 1, 2, 3, 4, 5] {
        let cancellation = crate::CancellationSource::new();
        let mut policy = cancellation_request(if backing >= 3 { backing - 1 } else { 1 });
        policy.cancellation = cancellation.token();
        let lease = match backing {
            0 => None,
            1 => Some(crate::ExecutionResourceLease::serial(policy).unwrap()),
            2..=5 => Some(physical.request_lease(policy).unwrap()),
            _ => unreachable!(),
        };
        let mut measured = None;
        let outcome = run_scope(lease.as_ref(), 0, 0, |context| {
            let plan = ReductionPlan::try_from_sorted_unique(
                [1, 3, 4].map(PartitionIdentity::new).to_vec(),
            )
            .unwrap();
            let combine = |left: &u64, right: &u64| {
                if *left == 4 && *right == 4 {
                    cancellation.cancel();
                }
                left + right
            };
            let before = context.cost();
            let result = if let Some(lease) = &lease {
                ReductionTree::build_scheduled_checked(
                    lease,
                    BackendKind::Serial,
                    plan,
                    vec![1_u64, 3, 4],
                    0,
                    combine,
                    8,
                    context,
                )
                .map_err(|error| match error {
                    ScheduledReductionError::Reduction(failure) => failure,
                    ScheduledReductionError::Admission(denial) => panic!("admission: {denial:?}"),
                })
            } else {
                ReductionTree::from_plan_checked(plan, vec![1_u64, 3, 4], 0, combine, 8, || {
                    if cancellation.token().is_cancelled() {
                        Err(KernelStop::Cancelled)
                    } else {
                        context.checkpoint(1)
                    }
                })
            };
            let metrics = match result {
                Ok((_, metrics)) => {
                    if lease.is_none() {
                        context
                            .apply_structural_span(
                                before,
                                metrics.accounted_metrics().unwrap().charged_work,
                                metrics.accounted_metrics().unwrap().charged_span,
                            )
                            .unwrap();
                    }
                    metrics
                }
                Err(failure) => {
                    assert!(matches!(
                        failure.reason,
                        ReductionRunStop::Hook(KernelStop::Cancelled)
                    ));
                    failure.metrics
                }
            };
            assert_eq!(metrics.accounted_metrics().unwrap().combine_calls, 2 * 3);
            assert_eq!(metrics.accounted_metrics().unwrap().recombined_nodes, 3);
            measured = Some(metrics);
            Ok::<(), KernelFailure<()>>(())
        });
        let metrics = measured.unwrap();
        assert_eq!(
            outcome.report.charged_work(),
            metrics.accounted_metrics().unwrap().charged_work
        );
        assert_eq!(
            outcome.report.charged_span(),
            metrics.accounted_metrics().unwrap().charged_span
        );
        let observed = (
            metrics,
            outcome.report.charged_work(),
            outcome.report.charged_span(),
            matches!(outcome.result, Ok(())),
        );
        assert!(
            observed.3,
            "no checkpoint follows the final canonical combine"
        );
        if let Some(expected) = &oracle {
            assert_eq!(&observed, expected, "backing {backing}");
        } else {
            oracle = Some(observed);
        }
    }
}

// Each cancellation fixture runs in its own process because the physical
// authority is process-unique. No memory forecast gates its numerical oracle.
fn cancellation_authority() -> crate::ExecutionAuthority {
    crate::ExecutionAuthority::try_construct(crate::ExecutionAuthorityConfig {
        max_workers: std::num::NonZeroUsize::new(4).unwrap(),
        charged_memory_bytes: None,
    })
    .unwrap()
}

fn cancellation_request(workers: usize) -> crate::LeaseRequest {
    crate::LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Automatic,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::new(workers).unwrap(),
                None,
                Some(100),
            ),
        ),
        cancellation: crate::CancellationToken::new(),
        deadline: None,
    }
}
