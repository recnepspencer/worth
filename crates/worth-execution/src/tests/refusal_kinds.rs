//! Only questions handed to kernels may influence their later failure owner.
use super::*;
use crate::{NestedComputationStop, PartitionWork};

#[test]
fn oracle_reservation_failure_settles_before_its_caller_can_return_success() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = authority().request_lease(request(1, 100, 100)).unwrap();
    let declared =
        AdmittedBatch::try_admit(vec![(PartitionIdentity::new(1), 1_u64, 0, 100)], 0).unwrap();
    let outer = crate::backend::run_scope(None, 0, 0, |parent| {
        let inner = crate::backend::run_scope(None, 0, 0, |context| {
            let result =
                crate::oracle::certify(&lease, &declared, BackendKind::Serial, &|_,
                                                                                 _|
                 -> Result<
                    u64,
                    KernelFailure<()>,
                > {
                    panic!("reservation fails before either kernel")
                });
            assert!(matches!(result, Err(crate::oracle::OracleMismatch::Stop)));
            assert_eq!(context.take_nested_computation_stop(), None);
            assert_eq!(context.checkpoint(0), Err(KernelStop::NestedStopped));
            Ok::<_, KernelFailure<()>>(())
        });
        assert!(inner.result.is_err());
        assert_eq!(parent.take_nested_computation_stop(), None);
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(outer.result.is_err());
    assert_eq!(
        (outer.report.charged_work(), outer.report.charged_span()),
        (0, 0)
    );
}

#[test]
fn successful_certification_does_not_clear_an_earlier_kernel_question() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = authority().request_lease(request(1, 1_000, 100)).unwrap();
    let outer = crate::backend::run_scope(None, 0, 0, |parent| {
        let inner = crate::backend::run_scope(None, 0, 0, |_| {
            assert!(lease.reserve_memory(u64::MAX).is_err());
            let result = crate::oracle::certify(
                &lease,
                &batch(&[1], 0),
                BackendKind::Serial,
                &|value, _| Ok::<_, KernelFailure<()>>(*value),
            )
            .unwrap();
            assert_eq!(result.stop, None);
            Err::<(), _>(KernelFailure::Domain(()))
        });
        assert!(inner.result.is_err());
        assert_eq!(parent.take_nested_computation_stop(), None);
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(outer.result.is_err());
}

#[test]
fn size_overflow_is_request_owned_but_pure_policy_conversion_remains_takeable() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = authority().request_lease(request(1, 1_000, 100)).unwrap();
    for size_overflow in [false, true] {
        let outer = crate::backend::run_scope(None, 0, 0, |parent| {
            let inner = crate::backend::run_scope(None, 0, 0, |_| {
                if size_overflow {
                    let declared = AdmittedBatch::try_admit(
                        vec![(PartitionIdentity::new(1), 1_u64, 0, u64::MAX)],
                        0,
                    )
                    .unwrap();
                    let result = crate::oracle::certify(
                        &lease,
                        &declared,
                        BackendKind::Serial,
                        &|_, _| -> Result<u64, KernelFailure<()>> {
                            panic!("declared size fails before either kernel")
                        },
                    );
                    assert!(matches!(result, Err(crate::oracle::OracleMismatch::Stop)));
                } else {
                    let denial: LeaseDenial =
                        crate::ExecutionPolicyDenial::WorkersExceedAuthority.into();
                    assert_eq!(denial, LeaseDenial::WorkerLimitExceedsParent);
                }
                Err::<(), _>(KernelFailure::Domain(()))
            });
            assert!(inner.result.is_err());
            assert_eq!(
                parent.take_nested_computation_stop(),
                if size_overflow {
                    None
                } else {
                    Some(NestedComputationStop::DomainFailure)
                }
            );
            Ok::<_, KernelFailure<()>>(())
        });
        assert_eq!(outer.result.is_err(), size_overflow);
    }
}

#[test]
fn handed_policy_denial_owns_a_later_domain_failure() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let outer = crate::backend::run_scope(None, 0, 0, |parent| {
        let inner = crate::backend::run_scope(None, 0, 0, |_| {
            assert!(matches!(
                authority().request_lease(request(5, 1_000, 100)),
                Err(LeaseDenial::WorkerLimitExceedsParent)
            ));
            Err::<(), _>(KernelFailure::Domain(()))
        });
        assert!(inner.result.is_err());
        assert_eq!(parent.take_nested_computation_stop(), None);
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(outer.result.is_err());
}

#[test]
fn inconsistent_structural_metrics_are_a_takeable_computation_failure() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let outer = crate::backend::run_scope(None, 0, 0, |parent| {
        let inner = crate::backend::run_scope(None, 0, 0, |context| {
            context.checkpoint(2)?;
            let failure = context.apply_structural_span((0, 0), 2, 3).unwrap_err();
            assert_eq!(
                failure,
                crate::backend::StructuralSpanFailure::MetricsMismatch
            );
            let failure = crate::ReductionRunFailure::structural_span_failure(
                failure,
                crate::ReductionMetrics {
                    charged_work: 2,
                    charged_span: 3,
                    ..Default::default()
                },
            );
            assert_eq!(
                failure.reason,
                crate::ReductionRunStop::Denial(crate::ReductionDenial::CoverageMismatch)
            );
            Err::<(), _>(KernelFailure::Domain(failure))
        });
        assert!(inner.result.is_err());
        assert_eq!(
            parent.take_nested_computation_stop(),
            Some(NestedComputationStop::DomainFailure)
        );
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(outer.result.is_ok());
    assert_eq!(
        (outer.report.charged_work(), outer.report.charged_span()),
        (2, 2)
    );
}

#[test]
fn absorbed_busy_slots_do_not_own_the_launching_kernels_domain_failure() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let occupied = authority().request_lease(request(4, 100, 100)).unwrap();
    let mut serial = None;
    for workers in [1, 4] {
        let held = (workers == 4).then(|| occupied.try_reserve(4, 0).unwrap());
        let grandparent = authority()
            .request_lease(request(workers, 2_000, 100))
            .unwrap();
        let parent = grandparent.child(request(workers, 2_000, 100)).unwrap();
        let child = parent.child(request(workers, 2_000, 100)).unwrap();
        let mut returned_failure = None;
        let outer = crate::backend::run_scope(Some(&grandparent), 0, 0, |context| {
            let failed = run_checked_batch(
                Some(&parent),
                &batch(&[1], 0),
                BackendKind::Native,
                &|_, context| {
                    context.checkpoint(1)?;
                    let nested = run_checked_batch(
                        Some(&child),
                        &batch(&[1, 2], 0),
                        BackendKind::Native,
                        &|value, context| {
                            context.checkpoint(1)?;
                            Ok::<_, KernelFailure<()>>(*value)
                        },
                    );
                    assert_eq!(nested.stop, None);
                    assert_eq!(nested.values, vec![1, 2]);
                    assert_eq!(nested.report.physical().active_workers_high_watermark(), 1);
                    Err::<u64, _>(KernelFailure::Domain(7_u64))
                },
            );
            assert_eq!(
                failed.stop,
                Some(BatchStop::Failure {
                    identity: PartitionIdentity::new(1),
                    cause: KernelFailure::Domain(7_u64),
                })
            );
            assert_eq!(
                context.take_nested_computation_stop(),
                Some(NestedComputationStop::DomainFailure)
            );
            returned_failure = failed.stop;
            context.checkpoint(1)?;
            Ok::<_, KernelFailure<()>>(7_u64)
        });
        let actual = (
            returned_failure,
            outer.result.unwrap(),
            outer.report.charged_work(),
            outer.report.charged_span(),
        );
        assert_eq!(
            actual,
            (
                Some(BatchStop::Failure {
                    identity: PartitionIdentity::new(1),
                    cause: KernelFailure::Domain(7)
                }),
                7,
                4,
                3
            )
        );
        if let Some(expected) = &serial {
            assert_eq!(&actual, expected);
        } else {
            serial = Some(actual);
        }
        drop(held);
    }
}

#[test]
fn swallowed_partition_work_overflow_is_an_immediate_request_fact() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let parent = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let child = parent.child(request(1, 1_000, 100)).unwrap();
    let outer = run_checked_batch(
        Some(&parent),
        &batch(&[1], 0),
        BackendKind::Serial,
        &|_, context| {
            let inner = run_checked_batch(
                Some(&child),
                &batch(&[1], 0),
                BackendKind::Serial,
                &|_, context| {
                    let work = PartitionWork {
                        items_rerouted: u64::MAX,
                        members_visited: 1,
                        ..Default::default()
                    };
                    assert_eq!(work.charge(context), Err(KernelStop::WorkCounterOverflow));
                    assert_eq!(context.take_nested_computation_stop(), None);
                    Ok::<_, KernelFailure<()>>(1_u64)
                },
            );
            assert_eq!(
                inner.stop,
                Some(BatchStop::Failure {
                    identity: PartitionIdentity::new(1),
                    cause: KernelFailure::Stop(KernelStop::NestedStopped)
                })
            );
            assert_eq!(context.take_nested_computation_stop(), None);
            Ok::<_, KernelFailure<()>>(1_u64)
        },
    );
    assert_eq!(
        outer.stop,
        Some(BatchStop::Failure {
            identity: PartitionIdentity::new(1),
            cause: KernelFailure::Stop(KernelStop::NestedStopped)
        })
    );
    assert_eq!(
        (outer.report.charged_work(), outer.report.charged_span()),
        (0, 0)
    );
}
