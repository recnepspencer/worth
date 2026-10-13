//! One latch keeps the first computational cause until its kernel takes it.
use super::*;
use crate::NestedComputationStop;

#[test]
fn two_failing_children_keep_the_first_cause_at_every_width() {
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0_usize, 1, 2, 4] {
        for panic_first in [false, true] {
            let parent = authority()
                .request_lease(request(width.max(1), 2_000, 100))
                .unwrap();
            let child = parent.child(request(1, 1_000, 100)).unwrap();
            let backend = if width == 0 {
                BackendKind::Serial
            } else {
                BackendKind::Native
            };
            let outer =
                run_checked_batch(Some(&parent), &batch(&[1], 0), backend, &|_, context| {
                    for panics in [panic_first, !panic_first] {
                        let nested = run_checked_batch(
                            Some(&child),
                            &batch(&[1], 0),
                            backend,
                            &|_, context| {
                                context.checkpoint(2)?;
                                assert!(!panics, "declared child panic");
                                Err::<u64, _>(KernelFailure::Domain(()))
                            },
                        );
                        assert!(nested.stop.is_some());
                    }
                    let first = if panic_first {
                        NestedComputationStop::Panic
                    } else {
                        NestedComputationStop::DomainFailure
                    };
                    assert_eq!(context.take_nested_computation_stop(), Some(first));
                    assert_eq!(context.take_nested_computation_stop(), None);
                    Ok::<_, KernelFailure<()>>(first)
                });
            assert_eq!(outer.stop, None);
            assert_eq!(
                outer.values,
                vec![if panic_first {
                    NestedComputationStop::Panic
                } else {
                    NestedComputationStop::DomainFailure
                }]
            );
            assert_eq!(outer.report.charged_work(), 2 + 2);
            assert_eq!(outer.report.charged_span(), 2 + 2);
        }
    }
}

#[test]
fn concurrent_failing_children_return_the_serial_failure_work_and_span() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    for panic_first in [false, true] {
        let mut serial = None;
        for width in [0_usize, 2, 4] {
            let parent = authority()
                .request_lease(request(width.max(1), 2_000, 100))
                .unwrap();
            let child = parent.child(request(width.max(1), 2_000, 100)).unwrap();
            let backend = if width == 0 {
                BackendKind::Serial
            } else {
                BackendKind::Native
            };
            let overlap = std::sync::Barrier::new(2);
            let first = if panic_first {
                NestedComputationStop::Panic
            } else {
                NestedComputationStop::DomainFailure
            };
            let outer =
                run_checked_batch(Some(&parent), &batch(&[1], 0), backend, &|_, context| {
                    let nested = run_checked_batch(
                        Some(&child),
                        &batch(&[1, 2], 0),
                        backend,
                        &|value, context| {
                            context.checkpoint(*value + 1)?;
                            if width != 0 {
                                overlap.wait();
                            }
                            assert!((*value == 1) != panic_first, "declared child panic");
                            Err::<u64, _>(KernelFailure::Domain(*value))
                        },
                    );
                    let expected = if panic_first {
                        KernelFailure::Panic
                    } else {
                        KernelFailure::Domain(1_u64)
                    };
                    assert_eq!(
                        nested.stop,
                        Some(BatchStop::Failure {
                            identity: PartitionIdentity::new(1),
                            cause: expected
                        })
                    );
                    assert_eq!(
                        (nested.report.charged_work(), nested.report.charged_span()),
                        (2, 2)
                    );
                    assert_eq!(context.take_nested_computation_stop(), Some(first));
                    assert_eq!(context.take_nested_computation_stop(), None);
                    Ok::<_, KernelFailure<()>>(first)
                });
            let actual = (
                outer.stop,
                outer.values,
                outer.report.charged_work(),
                outer.report.charged_span(),
            );
            if let Some(expected) = &serial {
                assert_eq!(&actual, expected);
            } else {
                serial = Some(actual);
            }
        }
    }
}
