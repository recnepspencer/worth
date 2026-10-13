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
fn checkpoint_and_completed_counter_overflow_remain_request_owned() {
    let _serial = TEST_LOCK.lock().unwrap();
    for width in [0_usize, 1, 2, 4] {
        for completed in [false, true] {
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
                            if !completed {
                                context.checkpoint(1)?;
                            }
                            Ok::<_, KernelFailure<()>>(1_u64)
                        },
                    );
                    if completed {
                        assert_eq!(first.stop, None);
                        // A real completed-work charge overflows the invoking
                        // meter; admission prevents a real joined child from
                        // claiming one unit beyond its inherited remaining work.
                        assert_eq!(
                            context.account_completed_work(1),
                            Err(KernelStop::WorkCounterOverflow)
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
                    // A local refused increment is sticky, while an inherited
                    // child request stop is reported by the nested latch.
                    cause: KernelFailure::Stop(if completed {
                        KernelStop::WorkCounterOverflow
                    } else {
                        KernelStop::NestedStopped
                    }),
                })
            );
            assert_eq!(outer.report.charged_work(), u64::MAX);
            assert_eq!(outer.report.charged_span(), u64::MAX);
        }
    }
}

#[test]
fn a_real_safe_point_cannot_make_cancellation_takeable_as_domain_data() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    for width in [0_usize, 1, 2, 4] {
        let parent = authority()
            .request_lease(request(width.max(1), 2_000, 100))
            .unwrap();
        let source = CancellationSource::new();
        let child = parent
            .child(LeaseRequest {
                cancellation: source.token(),
                ..request(1, 1_000, 100)
            })
            .unwrap();
        let outer = run_checked_batch(
            Some(&parent),
            &batch(&[1], 0),
            backend(width),
            &|_, context| {
                let nested = run_checked_batch(
                    Some(&child),
                    &batch(&[1], 0),
                    backend(width),
                    &|_, child| {
                        source.cancel();
                        assert_eq!(child.safe_point(), Err(KernelStop::Cancelled));
                        Err::<u64, _>(KernelFailure::Domain(()))
                    },
                );
                assert_eq!(
                    nested.stop,
                    Some(BatchStop::Failure {
                        identity: PartitionIdentity::new(1),
                        cause: KernelFailure::Domain(()),
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
        assert_eq!(
            (outer.report.charged_work(), outer.report.charged_span()),
            (0, 0)
        );
    }
}

#[test]
fn an_uncharged_scope_preserves_a_real_producers_request_owner() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let outer = crate::backend::run_scope(None, 0, 0, |parent| {
        let inner = crate::backend::run_admitting_scope(None, 0, 0, false, |_| {
            assert!(lease.reserve_memory(u64::MAX).is_err());
            Err::<(), _>(crate::backend::ScopeKernelFailure::Kernel(
                KernelFailure::Domain(()),
            ))
        });
        assert!(matches!(
            inner.result,
            Err(crate::backend::ScopeStop::Failure(
                KernelFailure::Domain(())
            ))
        ));
        assert_eq!(parent.take_nested_computation_stop(), None);
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(matches!(
        outer.result,
        Err(crate::backend::ScopeStop::Failure(KernelFailure::Stop(
            KernelStop::NestedStopped
        )))
    ));
    assert_eq!(
        (outer.report.charged_work(), outer.report.charged_span()),
        (0, 0)
    );
}

#[test]
fn an_uncharged_oracle_request_dominates_an_equal_domain_result() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let parent = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let child = parent.child(request(1, 1_000, 100)).unwrap();
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let outer = run_checked_batch(
        Some(&parent),
        &batch(&[1], 0),
        BackendKind::Serial,
        &|_, context| {
            let certified =
                crate::oracle::certify(&child, &batch(&[1], 0), BackendKind::Serial, &|_, _| {
                    if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                        assert!(child.reserve_memory(u64::MAX).is_err());
                    }
                    Err::<u64, _>(KernelFailure::Domain(1_u64))
                })
                .unwrap();
            assert_eq!(calls.load(Ordering::SeqCst), 2);
            assert_eq!(
                certified.stop,
                Some(BatchStop::Failure {
                    identity: PartitionIdentity::new(1),
                    cause: KernelFailure::Domain(1_u64)
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
