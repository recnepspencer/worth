use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
    time::{Duration, Instant},
};

use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionForkJoin, ExecutionResourceLease, ExecutionRounds, ForkChild, ForkJoinOutcome,
    LeaseRequest, MapDenial, MapKernelFailure, MapKernelStop, MapStop, RoundsOutcome,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
static TEST_LOCK: Mutex<()> = Mutex::new(());

fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: 65_536,
        })
        .unwrap()
    })
}

fn lease(
    posture: ExecutionPosture,
    work: u64,
    cancellation: CancellationToken,
) -> ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                posture,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), 65_536, work),
            ),
            deadline: None,
            cancellation,
        })
        .unwrap()
}

fn fork() -> ExecutionForkJoin<u64, u64> {
    let identities = [PartitionIdentity::new(1), PartitionIdentity::new(2)];
    ExecutionForkJoin::try_from_children(
        identities.to_vec(),
        identities
            .into_iter()
            .map(|identity| ForkChild {
                identity,
                input: identity.value(),
                read_keys: Vec::new(),
                write_keys: Vec::new(),
                kernel_scratch_bytes: 0,
                max_result_bytes: 0,
            })
            .collect(),
    )
    .unwrap()
}

#[test]
fn fork_children_must_prove_disjoint_access_before_dispatch() {
    let ids = [PartitionIdentity::new(1), PartitionIdentity::new(2)];
    let child = |identity, read_keys, write_keys| ForkChild {
        identity,
        input: identity.value(),
        read_keys,
        write_keys,
        kernel_scratch_bytes: 0,
        max_result_bytes: 0,
    };
    let overlapping = ExecutionForkJoin::try_from_children(
        ids.to_vec(),
        vec![
            child(ids[0], vec![], vec![7]),
            child(ids[1], vec![], vec![7]),
        ],
    );
    assert!(matches!(
        overlapping,
        Err(MapDenial::WriteSetOverlap { .. })
    ));

    let read_after_write = ExecutionForkJoin::try_from_children(
        ids.to_vec(),
        vec![
            child(ids[0], vec![7], vec![]),
            child(ids[1], vec![], vec![7]),
        ],
    );
    assert!(matches!(
        read_after_write,
        Err(MapDenial::ReadWriteConflict { .. })
    ));
}

fn recursive(lease: &ExecutionResourceLease<'_>, depth: u32) -> ForkJoinOutcome<u64, ()> {
    fork().run(Some(lease), |value, context| {
        context.checkpoint(1)?;
        if depth == 0 {
            return Ok::<_, MapKernelFailure<()>>(*value);
        }
        match recursive(lease, depth - 1) {
            ForkJoinOutcome::Complete { values, .. } => {
                Ok(*value + values.into_iter().sum::<u64>())
            }
            _ => Err(MapKernelFailure::Stop(MapKernelStop::NestedStopped)),
        }
    })
}

#[test]
fn recursive_join_has_canonical_values_work_and_span_at_both_postures() {
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
