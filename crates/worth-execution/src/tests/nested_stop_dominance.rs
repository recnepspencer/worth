//! Every contributing request stop dominates computational data at settlement.
use super::*;
use crate::backend::run_owned_batch_taking;

fn backend(width: usize) -> BackendKind {
    if width == 0 {
        BackendKind::Serial
    } else {
        BackendKind::Native
    }
}
#[test]
fn request_dominates_a_batch_in_either_canonical_order() {
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0_usize, 1, 2, 4] {
        for request_rank in [1, 2] {
            let parent = authority()
                .request_lease(request(width.max(1), 2_000, 100))
                .unwrap();
            let child = parent.child(request(1, 1_000, 100)).unwrap();
            let unrelated = authority().request_lease(request(1, 500, 100)).unwrap();
            let outer = run_checked_batch(
                Some(&parent),
                &batch(&[1], 0),
                backend(width),
                &|_, context| {
                    context.checkpoint(1)?;
                    let nested = run_checked_batch(
                        Some(&child),
                        &batch(&[1, 2], 0),
                        backend(width),
                        &|value, context| {
                            context.checkpoint(1)?;
                            if *value == request_rank {
                                let refused = run_checked_batch(
                                    Some(&unrelated),
                                    &batch(&[1], 0),
                                    backend(width),
                                    &|_, _| Ok::<_, KernelFailure<()>>(1_u64),
                                );
                                assert_eq!(
                                    refused.stop,
                                    Some(BatchStop::Admission(LeaseDenial::UnrelatedNestedLease))
                                );
                                Ok::<_, KernelFailure<u64>>(*value)
                            } else {
                                Err(KernelFailure::Domain(*value))
                            }
                        },
                    );
                    assert_eq!(nested.prefix_boundary, Some(PartitionIdentity::new(1)));
                    assert_eq!(nested.report.charged_work(), 1);
                    assert_eq!(
                        context.take_nested_computation_stop(),
                        None,
                        "the suffix request stop cannot disappear behind the least domain failure"
                    );
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
            assert_eq!(outer.report.charged_work(), 1 + 1);
            assert_eq!(outer.report.charged_span(), 1 + 1);
        }
    }
}
#[test]
fn request_dominates_two_child_calls_in_either_order() {
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0_usize, 1, 2, 4] {
        for request_first in [false, true] {
            let parent = authority()
                .request_lease(request(width.max(1), 2_000, 100))
                .unwrap();
            let child = parent.child(request(1, 1_000, 100)).unwrap();
            let unrelated = authority().request_lease(request(1, 500, 100)).unwrap();
            let outer = run_checked_batch(
                Some(&parent),
                &batch(&[1], 0),
                backend(width),
                &|_, context| {
                    for is_request in [request_first, !request_first] {
                        let nested = run_checked_batch(
                            Some(if is_request { &unrelated } else { &child }),
                            &batch(&[1], 0),
                            backend(width),
                            &|_, context| {
                                context.checkpoint(2)?;
                                Err::<u64, _>(KernelFailure::Domain(()))
                            },
                        );
                        assert!(nested.stop.is_some());
                    }
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
            assert_eq!(outer.report.charged_work(), 2);
            assert_eq!(outer.report.charged_span(), 2);
        }
    }
}
#[test]
fn destruction_panic_does_not_release_an_interruption() {
    struct PanickingInput;
    impl crate::ChargedBytes for PanickingInput {
        fn additional_charged_bytes(&self) -> u64 {
            0
        }
    }
    impl Drop for PanickingInput {
        fn drop(&mut self) {
            panic!("skipped input destructor");
        }
    }
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0_usize, 1, 2, 4] {
        for deadline in [false, true] {
            let parent = authority()
                .request_lease(request(width.max(1), 2_000, 100))
                .unwrap();
            let cancellation = CancellationSource::new();
            let child = parent
                .child(LeaseRequest {
                    cancellation: cancellation.token(),
                    deadline: deadline.then(|| Instant::now() - Duration::from_secs(1)),
                    ..request(1, 1_000, 100)
                })
                .unwrap();
            if !deadline {
                cancellation.cancel();
            }
            let outer = run_checked_batch(
                Some(&parent),
                &batch(&[1], 0),
                backend(width),
                &|_, context| {
                    let input = AdmittedBatch::try_admit(
                        vec![(PartitionIdentity::new(1), PanickingInput, 0, 0)],
                        0,
                    )
                    .unwrap();
                    let nested = run_owned_batch_taking(
                        Some(&child),
                        input,
                        backend(width),
                        None,
                        &|_, _| -> Result<u64, KernelFailure<()>> {
                            panic!("interrupted input cannot compute")
                        },
                    );
                    // The established public law reports the destructor's panic;
                    // ownership still includes the interruption it replaced.
                    assert_eq!(
                        nested.stop,
                        Some(BatchStop::Failure {
                            identity: PartitionIdentity::new(1),
                            cause: KernelFailure::Panic
                        })
                    );
                    assert_eq!(
                        context.take_nested_computation_stop(),
                        None,
                        "the destructor cannot make a request interruption takeable"
                    );
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
            assert_eq!(outer.report.charged_work(), 0);
            assert_eq!(outer.report.charged_span(), 0);
        }
    }
}
