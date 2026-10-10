//! Request interruptions and arithmetic overflow cannot be acknowledged as data.
use super::*;

fn backend(width: usize) -> BackendKind {
    if width == 0 {
        BackendKind::Serial
    } else {
        BackendKind::Native
    }
}

#[test]
fn cancellation_and_deadline_remain_request_owned() {
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0_usize, 1, 2, 4] {
        for deadline in [false, true] {
            let parent = authority()
                .request_lease(request(width.max(1), 2_000, 100))
                .unwrap();
            let cancellation = CancellationSource::new();
            let child = parent
                .child(LeaseRequest {
                    deadline: deadline.then(|| Instant::now() - Duration::from_secs(1)),
                    cancellation: cancellation.token(),
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
                    let nested = run_checked_batch(
                        Some(&child),
                        &batch(&[1], 0),
                        backend(width),
                        &|_, _| Ok::<_, KernelFailure<()>>(1_u64),
                    );
                    let stop = if deadline {
                        KernelStop::DeadlineElapsed
                    } else {
                        KernelStop::Cancelled
                    };
                    assert_eq!(
                        nested.stop,
                        Some(BatchStop::Failure {
                            identity: PartitionIdentity::new(1),
                            cause: KernelFailure::Stop(stop),
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
                    cause: KernelFailure::Stop(KernelStop::NestedStopped),
                })
            );
            assert_eq!(outer.report.charged_work(), 0);
            assert_eq!(outer.report.charged_span(), 0);
        }
    }
}

#[test]
fn checkpoint_and_joined_counter_overflow_remain_request_owned() {
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0_usize, 1, 2, 4] {
        for joined in [false, true] {
            let parent = authority()
                .request_lease(request(width.max(1), 2_000, u64::MAX))
                .unwrap();
            let child = parent.child(request(1, 1_000, u64::MAX)).unwrap();
            let outer = run_checked_batch(
                Some(&parent),
                &batch(&[1], 0),
                backend(width),
                &|_, context| {
                    let first = run_checked_batch(
                        Some(&child),
                        &batch(&[1], 0),
                        backend(width),
                        &|_, context| {
                            context.checkpoint(u64::MAX)?;
                            if !joined {
                                context.checkpoint(1)?;
                            }
                            Ok::<_, KernelFailure<()>>(1_u64)
                        },
                    );
                    if joined {
                        assert_eq!(first.stop, None);
                        // Admission prevents another real child from exceeding
                        // the parent's remaining ceiling. Exercise the joined
                        // counters directly with a valid one-unit report.
                        crate::backend::record_nested(
                            worth_foundational::ExecutionReport::new(
                                ExecutionPosture::Serial,
                                1,
                                1,
                                worth_foundational::ExecutionPhysicalReport::new(0, 0, None, 0, 0),
                            ),
                            None,
                        );
                    } else {
                        assert_eq!(
                            first.stop,
                            Some(BatchStop::Failure {
                                identity: PartitionIdentity::new(1),
                                cause: KernelFailure::Stop(KernelStop::WorkCounterOverflow),
                            })
                        );
                    }
                    assert_eq!(context.take_nested_computation_stop(), None);
                    Ok::<_, KernelFailure<()>>(1_u64)
                },
            );
            assert_eq!(
                outer.stop,
                Some(BatchStop::Failure {
                    identity: PartitionIdentity::new(1),
                    cause: KernelFailure::Stop(KernelStop::NestedStopped),
                })
            );
            assert_eq!(outer.report.charged_work(), u64::MAX);
            assert_eq!(outer.report.charged_span(), u64::MAX);
        }
    }
}
