//! Prepared value handoff respects the invoking owner's actual stop settlement.
use super::*;
use worth_execution::{ExecutionWorkCeiling, NestedComputationStop};

#[test]
fn active_preparation_retains_untaken_computation_causes_and_permits_explicit_take() {
    for take in [false, true] {
        let lease = ExecutionResourceLease::serial(grant(100, None)).unwrap();
        let request = ExecutionRequest::leased(&lease);
        request
            .run_native_with_checkpoint(|context| {
                let result = request.prepare_native(|_| {
                    assert_eq!(
                        request.prepare_native(|_| Err::<(), _>("original domain")),
                        Ok(Err("original domain"))
                    );
                    if take {
                        assert_eq!(
                            context.take_nested_computation_stop(),
                            Some(NestedComputationStop::DomainFailure)
                        );
                    }
                    Ok::<_, ()>(())
                });
                assert_eq!(
                    result,
                    if take {
                        Ok(Ok(()))
                    } else {
                        Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
                    }
                );
                if !take {
                    assert_eq!(
                        context.take_nested_computation_stop(),
                        Some(NestedComputationStop::DomainFailure)
                    );
                }
            })
            .unwrap();
    }
}

#[test]
fn swallowed_request_exhaustion_cannot_be_taken_or_handed_off_as_prepared_success() {
    let lease = ExecutionResourceLease::serial(grant(2, None)).unwrap();
    let request = ExecutionRequest::leased(&lease);
    let outer = request.run_native_with_checkpoint(|context| {
        let result = request.prepare_native(|_| {
            assert!(step(request, 3).is_err());
            assert_eq!(context.take_nested_computation_stop(), None);
            Ok::<_, ()>(())
        });
        assert_eq!(
            result,
            Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
        );
    });
    assert!(matches!(
        outer,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
    ));
}

#[test]
fn preparation_contains_direct_and_swallowed_panics_on_active_and_standalone_entries() {
    for active in [false, true] {
        let lease = ExecutionResourceLease::serial(grant(100, None)).unwrap();
        let request = ExecutionRequest::leased(&lease);
        let operation = || {
            assert_eq!(
                request.prepare_native::<(), ()>(|_| panic!("native preparation panic")),
                Err(WorkCeilingDenial::Panicked)
            );
        };
        if active {
            request
                .run_native_with_checkpoint(|context| {
                    operation();
                    assert_eq!(
                        context.take_nested_computation_stop(),
                        Some(NestedComputationStop::Panic)
                    );
                    let swallowed = request.prepare_native(|_| {
                        assert_eq!(
                            request
                                .prepare_native::<(), ()>(|_| panic!("nested preparation panic")),
                            Err(WorkCeilingDenial::Panicked)
                        );
                        Ok::<_, ()>(())
                    });
                    assert_eq!(
                        swallowed,
                        Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
                    );
                    assert_eq!(
                        context.take_nested_computation_stop(),
                        Some(NestedComputationStop::Panic)
                    );
                })
                .unwrap();
        } else {
            operation();
        }
    }
}

#[test]
fn an_already_latched_request_stop_dominates_a_later_preparation_panic() {
    let lease = ExecutionResourceLease::serial(grant(2, None)).unwrap();
    let request = ExecutionRequest::leased(&lease);
    let outer = request.run_native_with_checkpoint(|context| {
        let result = request.prepare_native::<(), ()>(|_| {
            assert_eq!(context.checkpoint(3), Err(MapKernelStop::WorkCeiling));
            panic!("later panic cannot replace the original request stop");
        });
        assert_eq!(
            result,
            Err(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling))
        );
        assert_eq!(context.take_nested_computation_stop(), None);
    });
    assert!(matches!(
        outer,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling))
    ));
}

#[test]
fn preparation_keeps_inherited_refusals_and_settles_only_its_own_questions() {
    let lease = ExecutionResourceLease::serial(grant(100, Some(16 << 10))).unwrap();
    let request = ExecutionRequest::leased(&lease);
    let outer = request.run_native_with_checkpoint(|context| {
        // A rejected question swallowed by a successful preparation is lawful.
        assert_eq!(
            request.prepare_native(|_| {
                assert!(lease.reserve_memory(16 << 10).is_err());
                Ok::<_, ()>(())
            }),
            Ok(Ok(()))
        );
        assert_eq!(context.take_nested_computation_stop(), None);
        let result = request.prepare_native(|_| {
            assert!(lease.reserve_memory(16 << 10).is_err());
            // This question belongs to the enclosing preparation. Successful
            // active reuse must not erase it before that owner fails.
            assert_eq!(request.prepare_native(|_| Ok::<_, ()>(())), Ok(Ok(())));
            Err::<(), _>("original typed failure")
        });
        assert_eq!(result, Ok(Err("original typed failure")));
        assert_eq!(context.take_nested_computation_stop(), None);
        assert_eq!(context.poll(), Err(MapKernelStop::NestedStopped));
    });
    assert!(matches!(
        outer,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
    ));
}

#[test]
fn stopped_prepared_values_are_dropped_and_their_original_reservations_are_released() {
    struct Prepared {
        dropped: std::sync::Arc<std::sync::atomic::AtomicBool>,
        _memory: worth_execution::ExecutionMemoryReservation,
    }
    impl Drop for Prepared {
        fn drop(&mut self) {
            self.dropped
                .store(true, std::sync::atomic::Ordering::SeqCst);
            panic!("contained prepared-value disposal");
        }
    }
    for active in [false, true] {
        let cancellation = CancellationSource::new();
        let mut policy = grant(100, Some(16 << 10));
        policy.cancellation = cancellation.token();
        let lease = ExecutionResourceLease::serial(policy).unwrap();
        let request = ExecutionRequest::leased(&lease);
        let dropped = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let prepare = || {
            let result = request.prepare_native(|_| {
                let memory = lease.reserve_memory(100).unwrap();
                cancellation.cancel();
                Ok::<_, ()>(Prepared {
                    dropped: dropped.clone(),
                    _memory: memory,
                })
            });
            assert!(matches!(
                result,
                Err(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
            ));
        };
        if active {
            let outer = request.run_native_with_checkpoint(|_| prepare());
            assert!(matches!(
                outer,
                Err(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
            ));
        } else {
            prepare();
        }
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
        drop(lease.reserve_memory(16 << 10).unwrap());
    }
}

#[test]
fn serial_preparation_keeps_the_same_finite_request_work_and_domain_failure() {
    let serial = worth_execution::SerialRequest::from_policy(
        &grant(6, None).policy,
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    assert_eq!(
        request.prepare_native(|_| {
            step(request, 3)?;
            step(request, 4)
        }),
        Ok(Err(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling)))
    );
    let (_, report) = request
        .run(ExecutionWorkCeiling::new(None), |_| {
            request
                .prepare_native(|_| {
                    step(request, 3)?;
                    step(request, 3)
                })
                .unwrap()
                .unwrap();
        })
        .unwrap();
    assert_eq!(report.accounted_cost(), Some((6, 6)));
}
