#[allow(unused_imports)]
pub(crate) use crate::fast::fork_join_rounds::*;

#[test]
fn recursive_join_has_canonical_values_work_and_span_at_both_postures() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let serial = lease(ExecutionPosture::Serial, 14, CancellationToken::new());
    let native = lease(ExecutionPosture::Automatic, 14, CancellationToken::new());
    let left = recursive(&serial, 2);
    let right = recursive(&native, 2);
    assert!(matches!(&left, ForkJoinOutcome::Complete { values, .. } if values == &vec![10, 11]));
    assert!(matches!(&right, ForkJoinOutcome::Complete { values, .. } if values == &vec![10, 11]));
    assert_eq!(left.report().charged_work(), 14);
    assert_eq!(right.report().charged_work(), 14);
    assert_eq!(left.report().charged_span(), 3);
    assert_eq!(right.report().charged_span(), 3);
}

#[test]
fn fork_uses_canonical_failure_and_inherited_limits() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let native = lease(ExecutionPosture::Automatic, 2, CancellationToken::new());
    let failure = fork().run(
        Some(&native),
        |value, context| -> Result<u64, MapKernelFailure<&'static str>> {
            context.checkpoint(1)?;
            if *value == 1 {
                Err(MapKernelFailure::Domain("first"))
            } else {
                panic!("later child panic")
            }
        },
    );
    assert!(matches!(failure, ForkJoinOutcome::Stopped {
        completed_prefix, boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::Domain("first"), .. }, ..
    } if completed_prefix.is_empty() && identity == PartitionIdentity::new(1)));

    let bounded = lease(ExecutionPosture::Automatic, 1, CancellationToken::new());
    let exhausted = recursive(&bounded, 1);
    assert!(matches!(exhausted, ForkJoinOutcome::Stopped { .. }));
    assert!(exhausted.report().charged_work() <= 1);

    let token = CancellationSource::new();
    token.cancel();
    let cancelled = lease(ExecutionPosture::Automatic, 10, token.token());
    let outcome = fork().run(Some(&cancelled), |_, _| {
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(outcome, ForkJoinOutcome::Stopped {
        boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::Stop(MapKernelStop::Cancelled), .. }, ..
    } if identity == PartitionIdentity::new(1)));
}

#[test]
fn rounds_publish_only_complete_images_and_type_nonconvergence() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let plan = ExecutionRounds::try_new(NonZeroUsize::new(5).unwrap()).unwrap();
    let serial = lease(ExecutionPosture::Serial, 5, CancellationToken::new());
    let native = lease(ExecutionPosture::Automatic, 5, CancellationToken::new());
    let run = |lease: &ExecutionResourceLease<'_>| {
        plan.run(
            Some(lease),
            0_u64,
            0,
            0,
            0,
            |prior, round, context| {
                context.checkpoint(1)?;
                assert_eq!(*prior, (round - 1) as u64);
                Ok::<_, MapKernelFailure<()>>(*prior + 1)
            },
            |_, next| *next == 3,
        )
    };
    let left = run(&serial);
    let right = run(&native);
    assert!(matches!(
        left,
        RoundsOutcome::Converged {
            state: 3,
            rounds: 3,
            ..
        }
    ));
    assert!(matches!(
        right,
        RoundsOutcome::Converged {
            state: 3,
            rounds: 3,
            ..
        }
    ));
    assert_eq!(left.report().charged_work(), 3);
    assert_eq!(right.report().charged_work(), 3);

    let never = plan.run(
        None,
        0_u64,
        0,
        0,
        0,
        |prior, _, context| {
            context.checkpoint(1)?;
            Ok::<_, MapKernelFailure<()>>(*prior + 1)
        },
        |_, _| false,
    );
    assert!(matches!(
        never,
        RoundsOutcome::NotConverged {
            state: 5,
            rounds: 5,
            ..
        }
    ));
    assert_eq!(never.report().charged_work(), 5);
}

#[test]
fn rounds_stop_at_canonical_barrier_on_failure_panic_and_limits() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let plan = ExecutionRounds::try_new(NonZeroUsize::new(4).unwrap()).unwrap();
    let failed: RoundsOutcome<u64, &'static str> = plan.run(
        None,
        0_u64,
        0,
        0,
        0,
        |prior, round, context| {
            context.checkpoint(1)?;
            if round == 3 {
                return Err(MapKernelFailure::Domain("bad round"));
            }
            Ok(*prior + 1)
        },
        |_, _| false,
    );
    assert!(matches!(failed, RoundsOutcome::Stopped {
        completed_state: 2, completed_rounds: 2, boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::Domain("bad round"), .. }, ..
    } if identity == PartitionIdentity::new(3)));

    let panicked = plan.run(
        None,
        0_u64,
        0,
        0,
        0,
        |prior, round, _| {
            if round == 2 {
                panic!("round panic");
            }
            Ok::<_, MapKernelFailure<()>>(*prior + 1)
        },
        |_, _| false,
    );
    assert!(matches!(
        panicked,
        RoundsOutcome::Stopped {
            completed_state: 1,
            completed_rounds: 1,
            reason: MapStop::Failure {
                cause: MapKernelFailure::Panic,
                ..
            },
            ..
        }
    ));

    let limited = lease(ExecutionPosture::Automatic, 2, CancellationToken::new());
    let ceiling = plan.run(
        Some(&limited),
        0_u64,
        0,
        0,
        0,
        |prior, _, context| {
            context.checkpoint(1)?;
            Ok::<_, MapKernelFailure<()>>(*prior + 1)
        },
        |_, _| false,
    );
    assert!(matches!(ceiling, RoundsOutcome::Stopped {
        completed_state: 2, completed_rounds: 2, boundary: Some(identity),
        reason: MapStop::WorkExhausted { .. }, ..
    } if identity == PartitionIdentity::new(3)));

    let token = CancellationSource::new();
    let cancelled_lease = lease(ExecutionPosture::Automatic, 4, token.token());
    let cancelled = plan.run(
        Some(&cancelled_lease),
        0_u64,
        0,
        0,
        0,
        |prior, round, context| {
            context.checkpoint(1)?;
            if round == 1 {
                token.cancel();
            }
            Ok::<_, MapKernelFailure<()>>(*prior + 1)
        },
        |_, _| false,
    );
    assert!(matches!(cancelled, RoundsOutcome::Stopped {
        completed_state: 1, completed_rounds: 1, boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::Stop(MapKernelStop::Cancelled), .. }, ..
    } if identity == PartitionIdentity::new(2)));
}

#[test]
fn ordered_rounds_admit_parallel_nested_forks_and_preserve_failure_barrier() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let plan = ExecutionRounds::try_new(NonZeroUsize::new(2).unwrap()).unwrap();
    let lease = lease(ExecutionPosture::Automatic, 4, CancellationToken::new());
    let active = AtomicUsize::new(0);
    let failed: RoundsOutcome<u64, &'static str> = plan.run(
        Some(&lease),
        0_u64,
        0,
        0,
        0,
        |prior, round, _| {
            let children = fork().run(Some(&lease), |value, context| {
                context.checkpoint(1)?;
                if round == 1 {
                    active.fetch_add(1, Ordering::SeqCst);
                    let deadline = Instant::now() + Duration::from_secs(5);
                    while active.load(Ordering::SeqCst) < 2 {
                        if Instant::now() >= deadline {
                            return Err(MapKernelFailure::Domain("nested fork ran serially"));
                        }
                        std::thread::yield_now();
                    }
                } else if *value == 1 {
                    return Err(MapKernelFailure::Domain("child failure"));
                }
                Ok::<_, MapKernelFailure<&'static str>>(*value)
            });
            match children {
                ForkJoinOutcome::Complete { values, .. } => {
                    Ok(*prior + values.into_iter().sum::<u64>())
                }
                _ => Ok(*prior + 100), // The ordered runner must reject this ignored nested failure.
            }
        },
        |_, _| false,
    );
    assert_eq!(failed.report().charged_work(), 3);
    assert_eq!(failed.report().charged_span(), 2);
    assert_eq!(
        failed.report().resolved_posture(),
        ExecutionPosture::Automatic
    );
    assert!(failed.report().physical().active_workers_high_watermark() >= 2);
    assert!(matches!(failed, RoundsOutcome::Stopped {
        completed_state: 3, completed_rounds: 1, boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::Stop(MapKernelStop::NestedStopped), .. }, ..
    } if identity == PartitionIdentity::new(2)));
}
