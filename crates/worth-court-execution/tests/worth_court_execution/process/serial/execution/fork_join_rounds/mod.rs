#[allow(unused_imports)]
pub(crate) use {std::sync::atomic::AtomicUsize, worth_execution::CancellationSource, worth_execution::CancellationToken, std::time::Duration, worth_foundational::ExecutionPosture, worth_execution::ExecutionResourceLease, worth_execution::ExecutionRounds, worth_execution::ForkJoinOutcome, std::time::Instant, worth_execution::MapKernelFailure, worth_execution::MapKernelStop, worth_execution::MapStop, std::num::NonZeroUsize, std::sync::atomic::Ordering, worth_foundational::PartitionIdentity, worth_execution::RoundsOutcome, crate::fork_join_rounds::ordinary_context::TEST_LOCK, crate::fork_join_rounds::ordinary_context::fork, crate::fork_join_rounds::ordinary_context::lease, crate::fork_join_rounds::ordinary_context::recursive};

#[test]
fn recursive_join_has_canonical_values_work_and_span_at_both_postures() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let serial = lease(worth_foundational::ExecutionPosture::Serial, 14, worth_execution::CancellationToken::new());
    let native = lease(worth_foundational::ExecutionPosture::Automatic, 14, worth_execution::CancellationToken::new());
    let left = recursive(&serial, 2);
    let right = recursive(&native, 2);
    assert!(matches!(&left, worth_execution::ForkJoinOutcome::Complete { values, .. } if values == &vec![10, 11]));
    assert!(matches!(&right, worth_execution::ForkJoinOutcome::Complete { values, .. } if values == &vec![10, 11]));
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
    let native = lease(worth_foundational::ExecutionPosture::Automatic, 2, worth_execution::CancellationToken::new());
    let failure = fork().run(
        Some(&native),
        |value, context| -> Result<u64, MapKernelFailure<&'static str>> {
            context.checkpoint(1)?;
            if *value == 1 {
                Err(worth_execution::MapKernelFailure::Domain("first"))
            } else {
                panic!("later child panic")
            }
        },
    );
    assert!(matches!(failure, worth_execution::ForkJoinOutcome::Stopped {
        completed_prefix, boundary: Some(identity),
        reason: worth_execution::MapStop::Failure { cause: worth_execution::MapKernelFailure::Domain("first"), .. }, ..
    } if completed_prefix.is_empty() && identity == worth_foundational::PartitionIdentity::new(1)));

    let bounded = lease(worth_foundational::ExecutionPosture::Automatic, 1, worth_execution::CancellationToken::new());
    let exhausted = recursive(&bounded, 1);
    assert!(matches!(exhausted, worth_execution::ForkJoinOutcome::Stopped { .. }));
    assert!(exhausted.report().charged_work() <= 1);

    let token = worth_execution::CancellationSource::new();
    token.cancel();
    let cancelled = lease(worth_foundational::ExecutionPosture::Automatic, 10, token.token());
    let outcome = fork().run(Some(&cancelled), |_, _| {
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(outcome, worth_execution::ForkJoinOutcome::Stopped {
        boundary: Some(identity),
        reason: worth_execution::MapStop::Failure { cause: worth_execution::MapKernelFailure::Stop(worth_execution::MapKernelStop::Cancelled), .. }, ..
    } if identity == worth_foundational::PartitionIdentity::new(1)));
}

#[test]
fn rounds_publish_only_complete_images_and_type_nonconvergence() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let plan = worth_execution::ExecutionRounds::try_new(std::num::NonZeroUsize::new(5).unwrap()).unwrap();
    let serial = lease(worth_foundational::ExecutionPosture::Serial, 5, worth_execution::CancellationToken::new());
    let native = lease(worth_foundational::ExecutionPosture::Automatic, 5, worth_execution::CancellationToken::new());
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
        worth_execution::RoundsOutcome::Converged {
            state: 3,
            rounds: 3,
            ..
        }
    ));
    assert!(matches!(
        right,
        worth_execution::RoundsOutcome::Converged {
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
        worth_execution::RoundsOutcome::NotConverged {
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
    let plan = worth_execution::ExecutionRounds::try_new(std::num::NonZeroUsize::new(4).unwrap()).unwrap();
    let failed: RoundsOutcome<u64, &'static str> = plan.run(
        None,
        0_u64,
        0,
        0,
        0,
        |prior, round, context| {
            context.checkpoint(1)?;
            if round == 3 {
                return Err(worth_execution::MapKernelFailure::Domain("bad round"));
            }
            Ok(*prior + 1)
        },
        |_, _| false,
    );
    assert!(matches!(failed, worth_execution::RoundsOutcome::Stopped {
        completed_state: 2, completed_rounds: 2, boundary: Some(identity),
        reason: worth_execution::MapStop::Failure { cause: worth_execution::MapKernelFailure::Domain("bad round"), .. }, ..
    } if identity == worth_foundational::PartitionIdentity::new(3)));

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
        worth_execution::RoundsOutcome::Stopped {
            completed_state: 1,
            completed_rounds: 1,
            reason: worth_execution::MapStop::Failure {
                cause: worth_execution::MapKernelFailure::Panic,
                ..
            },
            ..
        }
    ));

    let limited = lease(worth_foundational::ExecutionPosture::Automatic, 2, worth_execution::CancellationToken::new());
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
    assert!(matches!(ceiling, worth_execution::RoundsOutcome::Stopped {
        completed_state: 2, completed_rounds: 2, boundary: Some(identity),
        reason: worth_execution::MapStop::WorkExhausted { .. }, ..
    } if identity == worth_foundational::PartitionIdentity::new(3)));

    let token = worth_execution::CancellationSource::new();
    let cancelled_lease = lease(worth_foundational::ExecutionPosture::Automatic, 4, token.token());
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
    assert!(matches!(cancelled, worth_execution::RoundsOutcome::Stopped {
        completed_state: 1, completed_rounds: 1, boundary: Some(identity),
        reason: worth_execution::MapStop::Failure { cause: worth_execution::MapKernelFailure::Stop(worth_execution::MapKernelStop::Cancelled), .. }, ..
    } if identity == worth_foundational::PartitionIdentity::new(2)));
}

#[test]
fn ordered_rounds_admit_parallel_nested_forks_and_preserve_failure_barrier() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let plan = worth_execution::ExecutionRounds::try_new(std::num::NonZeroUsize::new(2).unwrap()).unwrap();
    let lease = lease(worth_foundational::ExecutionPosture::Automatic, 4, worth_execution::CancellationToken::new());
    let active = std::sync::atomic::AtomicUsize::new(0);
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
                    active.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                    while active.load(std::sync::atomic::Ordering::SeqCst) < 2 {
                        if std::time::Instant::now() >= deadline {
                            return Err(worth_execution::MapKernelFailure::Domain("nested fork ran serially"));
                        }
                        std::thread::yield_now();
                    }
                } else if *value == 1 {
                    return Err(worth_execution::MapKernelFailure::Domain("child failure"));
                }
                Ok::<_, MapKernelFailure<&'static str>>(*value)
            });
            match children {
                worth_execution::ForkJoinOutcome::Complete { values, .. } => {
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
        worth_foundational::ExecutionPosture::Automatic
    );
    assert!(failed.report().physical().active_workers_high_watermark() >= 2);
    assert!(matches!(failed, worth_execution::RoundsOutcome::Stopped {
        completed_state: 3, completed_rounds: 1, boundary: Some(identity),
        reason: worth_execution::MapStop::Failure { cause: worth_execution::MapKernelFailure::Stop(worth_execution::MapKernelStop::NestedStopped), .. }, ..
    } if identity == worth_foundational::PartitionIdentity::new(2)));
}
