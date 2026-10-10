//! Nested failure belongs either to the computation or irreversibly to its request.
use super::*;
use crate::{ExecutionWorkCeiling, NestedComputationStop};

fn placement(width: usize) -> BackendKind {
    if width == 0 {
        BackendKind::Serial
    } else {
        BackendKind::Native
    }
}

#[test]
fn taken_domain_failure_is_data_and_every_outer_member_completes() {
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0, 1, 2, 4] {
        let parent = authority()
            .request_lease(request(width.max(1), 2_000, 100))
            .unwrap();
        let child = parent.child(request(1, 1_000, 100)).unwrap();
        let outer = run_checked_batch(
            Some(&parent),
            &batch(&[1, 2], 0),
            placement(width),
            &|value, context| {
                context.checkpoint(1)?;
                let nested = run_checked_batch(
                    Some(&child),
                    &batch(&[*value], 0),
                    placement(width),
                    &|value, child| {
                        child.checkpoint(2)?;
                        if *value == 1 {
                            Err::<u64, _>(KernelFailure::Domain(*value))
                        } else {
                            Ok(*value)
                        }
                    },
                );
                assert_eq!(
                    context.take_nested_computation_stop(),
                    (*value == 1).then_some(NestedComputationStop::DomainFailure)
                );
                assert_eq!(context.take_nested_computation_stop(), None);
                let cause = match nested.stop {
                    Some(BatchStop::Failure {
                        cause: KernelFailure::Domain(cause),
                        ..
                    }) => cause,
                    None => nested.values[0],
                    other => panic!(
                        "the child returns its domain data or its successful value: {other:?}"
                    ),
                };
                Ok::<_, KernelFailure<()>>(cause)
            },
        );
        assert_eq!(outer.stop, None);
        assert_eq!(outer.values, vec![1, 2]);
        assert_eq!(outer.report.charged_work(), 2 * (1 + 2));
        assert_eq!(outer.report.charged_span(), 1 + 2);
    }
}

#[test]
fn a_request_owned_child_refusal_cannot_be_taken() {
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0, 1, 2, 4] {
        let parent = authority()
            .request_lease(request(width.max(1), 2_000, 100))
            .unwrap();
        let unrelated = authority().request_lease(request(1, 500, 100)).unwrap();
        let outer = run_checked_batch(
            Some(&parent),
            &batch(&[1, 2], 0),
            placement(width),
            &|_, context| {
                context.checkpoint(1)?;
                let nested = run_checked_batch(
                    Some(&unrelated),
                    &batch(&[1], 0),
                    placement(width),
                    &|_, _| Ok::<_, KernelFailure<()>>(1_u64),
                );
                assert_eq!(
                    nested.stop,
                    Some(BatchStop::Admission(LeaseDenial::UnrelatedNestedLease))
                );
                assert_eq!(context.take_nested_computation_stop(), None);
                Ok::<_, KernelFailure<()>>(1_u64)
            },
        );
        assert_eq!(outer.values, Vec::<u64>::new());
        assert_eq!(
            outer.stop,
            Some(BatchStop::Failure {
                identity: PartitionIdentity::new(1),
                cause: KernelFailure::Stop(KernelStop::NestedStopped)
            })
        );
    }
}

#[test]
fn an_untaken_cause_stops_the_parent_without_changing_its_cost() {
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0, 1, 2, 4] {
        let parent = authority()
            .request_lease(request(width.max(1), 2_000, 100))
            .unwrap();
        let child = parent.child(request(1, 1_000, 100)).unwrap();
        let run = |take| {
            run_checked_batch(
                Some(&parent),
                &batch(&[1], 0),
                placement(width),
                &|_, context| {
                    context.checkpoint(1)?;
                    let nested = run_checked_batch(
                        Some(&child),
                        &batch(&[1], 0),
                        placement(width),
                        &|_, context| {
                            context.checkpoint(2)?;
                            Err::<u64, _>(KernelFailure::Domain(()))
                        },
                    );
                    assert!(nested.stop.is_some());
                    if take {
                        assert_eq!(
                            context.take_nested_computation_stop(),
                            Some(NestedComputationStop::DomainFailure)
                        );
                    }
                    Ok::<_, KernelFailure<()>>(1_u64)
                },
            )
        };
        let taken = run(true);
        let untaken = run(false);
        assert_eq!(taken.stop, None);
        assert_eq!(
            untaken.stop,
            Some(BatchStop::Failure {
                identity: PartitionIdentity::new(1),
                cause: KernelFailure::Stop(KernelStop::NestedStopped)
            })
        );
        assert_eq!(taken.report.charged_work(), 1 + 2);
        assert_eq!(taken.report.charged_span(), 1 + 2);
        assert_eq!(taken.report.charged_work(), untaken.report.charged_work());
        assert_eq!(taken.report.charged_span(), untaken.report.charged_span());
    }
}

#[test]
fn only_a_strictly_narrower_declared_ceiling_can_be_taken() {
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0, 1, 2, 4] {
        for computation_owned in [false, true] {
            let limit = if computation_owned { 100 } else { 1 };
            let parent = authority()
                .request_lease(request(width.max(1), 2_000, limit))
                .unwrap();
            let child = parent.child(request(1, 1_000, limit)).unwrap();
            let outer = run_checked_batch(
                Some(&parent),
                &batch(&[1], 0),
                placement(width),
                &|_, context| {
                    let (nested, _) = ExecutionWorkCeiling::new(1)
                        .run(&child, || {
                            run_checked_batch(
                                Some(&child),
                                &batch(&[1], 0),
                                placement(width),
                                &|_, context| {
                                    context.checkpoint(2)?;
                                    Ok::<_, KernelFailure<()>>(1_u64)
                                },
                            )
                        })
                        .unwrap();
                    assert_eq!(
                        nested.stop,
                        Some(BatchStop::WorkExhausted {
                            identity: PartitionIdentity::new(1)
                        })
                    );
                    assert_eq!(
                        context.take_nested_computation_stop(),
                        computation_owned.then_some(NestedComputationStop::WorkCeiling)
                    );
                    Ok::<_, KernelFailure<()>>(1_u64)
                },
            );
            assert_eq!(outer.stop.is_none(), computation_owned);
            assert_eq!(outer.report.charged_work(), 0);
            assert_eq!(outer.report.charged_span(), 0);
        }
    }
}

#[test]
fn a_checkpoint_cannot_release_any_untaken_cause_or_a_request_stop() {
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0, 1, 2, 4] {
        let parent = authority()
            .request_lease(request(width.max(1), 2_000, 100))
            .unwrap();
        let child = parent.child(request(1, 1_000, 100)).unwrap();
        let unrelated = authority().request_lease(request(1, 500, 100)).unwrap();
        let outer = run_checked_batch(
            Some(&parent),
            &batch(&[1], 0),
            placement(width),
            &|_, context| {
                for _ in 0..1 {
                    let nested = run_checked_batch(
                        Some(&child),
                        &batch(&[1], 0),
                        placement(width),
                        &|_, child| {
                            child.checkpoint(1)?;
                            Err::<u64, _>(KernelFailure::Domain(()))
                        },
                    );
                    assert!(matches!(
                        nested.stop,
                        Some(BatchStop::Failure {
                            cause: KernelFailure::Domain(()),
                            ..
                        })
                    ));
                }
                assert_eq!(context.checkpoint(0), Err(KernelStop::NestedStopped));
                assert_eq!(
                    context.take_nested_computation_stop(),
                    Some(NestedComputationStop::DomainFailure)
                );
                context.checkpoint(0)?;
                let nested = run_checked_batch(
                    Some(&unrelated),
                    &batch(&[1], 0),
                    placement(width),
                    &|_, _| Ok::<_, KernelFailure<()>>(1_u64),
                );
                assert!(matches!(
                    nested.stop,
                    Some(BatchStop::Admission(LeaseDenial::UnrelatedNestedLease))
                ));
                assert_eq!(context.take_nested_computation_stop(), None);
                Ok::<_, KernelFailure<()>>(1_u64)
            },
        );
        assert!(matches!(
            outer.stop,
            Some(BatchStop::Failure {
                cause: KernelFailure::Stop(KernelStop::NestedStopped),
                ..
            })
        ));
        assert_eq!(outer.report.charged_work(), 1);
    }
}

#[test]
fn child_panic_and_result_capacity_are_computation_owned() {
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0, 1, 2, 4] {
        for panics in [false, true] {
            let parent = authority()
                .request_lease(request(width.max(1), 2_000, 100))
                .unwrap();
            let child = parent.child(request(1, 1_000, 100)).unwrap();
            let outer = run_checked_batch(
                Some(&parent),
                &batch(&[1], 0),
                placement(width),
                &|_, context| {
                    let nested = run_checked_batch(
                        Some(&child),
                        &batch(&[1], 0),
                        placement(width),
                        &|_, context| {
                            context.checkpoint(2)?;
                            assert!(!panics, "declared child panic");
                            Ok::<_, KernelFailure<()>>(vec![1_u64])
                        },
                    );
                    let (failure, taken) = if panics {
                        (KernelFailure::Panic, NestedComputationStop::Panic)
                    } else {
                        (
                            KernelFailure::ResultCapacityExceeded,
                            NestedComputationStop::ResultCapacityExceeded,
                        )
                    };
                    assert_eq!(
                        nested.stop,
                        Some(BatchStop::Failure {
                            identity: PartitionIdentity::new(1),
                            cause: failure
                        })
                    );
                    assert_eq!(context.take_nested_computation_stop(), Some(taken));
                    Ok::<_, KernelFailure<()>>(1_u64)
                },
            );
            assert_eq!(outer.stop, None);
            assert_eq!(outer.report.charged_work(), 2);
            assert_eq!(outer.report.charged_span(), 2);
        }
    }
}
