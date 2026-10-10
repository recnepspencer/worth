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
