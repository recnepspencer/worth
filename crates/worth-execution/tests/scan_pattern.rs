use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
};

use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionScan, LeaseDenial,
    LeaseRequest, MapKernelFailure, MapKernelStop, MapStop, ScanOutcome,
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
            max_workers: NonZeroUsize::new(2).unwrap(),
            charged_memory_bytes: 4096,
        })
        .unwrap()
    })
}

fn lease(
    work: u64,
    cancellation: CancellationToken,
) -> worth_execution::ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), 4096, work),
            ),
            deadline: None,
            cancellation,
        })
        .unwrap()
}

fn scan() -> ExecutionScan<u64> {
    let ids = [1, 2, 3].map(PartitionIdentity::new);
    ExecutionScan::try_from_ordered(ids.to_vec(), ids.into_iter().zip([1, 2, 3]).collect()).unwrap()
}

#[test]
fn ordered_carry_and_prefixes_are_deterministic() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = lease(3, CancellationToken::new());
    let outcome = scan().run(Some(&lease), 0_u64, 0, 0, 0, 0, |carry, item, context| {
        context.checkpoint(1)?;
        let next = *carry + *item;
        Ok::<_, MapKernelFailure<()>>((next, next))
    });
    assert!(
        matches!(&outcome, ScanOutcome::Complete { state: 6, prefixes, .. } if prefixes == &vec![1, 3, 6])
    );
    assert_eq!(outcome.report().charged_work(), 3);
    assert_eq!(outcome.report().charged_span(), 3);
    assert_eq!(
        outcome.report().resolved_posture(),
        ExecutionPosture::Serial
    );
}

#[test]
fn failed_step_keeps_only_the_committed_prefix() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let outcome = scan().run(None, 0_u64, 0, 0, 0, 0, |carry, item, context| {
        context.checkpoint(1)?;
        if *item == 2 {
            return Err(MapKernelFailure::Domain("invalid"));
        }
        let next = *carry + *item;
        Ok((next, next))
    });
    assert!(matches!(outcome, ScanOutcome::Stopped {
        completed_state: 1,
        completed_prefix,
        boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::Domain("invalid"), .. },
        ..
    } if completed_prefix == vec![1] && identity == PartitionIdentity::new(2)));
}

#[test]
fn work_ceiling_and_cancellation_stop_at_named_carry_boundaries() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let first_lease = lease(2, CancellationToken::new());
    let exhausted = scan().run(
        Some(&first_lease),
        0_u64,
        0,
        0,
        0,
        0,
        |carry, item, context| {
            context.checkpoint(1)?;
            let next = *carry + *item;
            Ok::<_, MapKernelFailure<()>>((next, next))
        },
    );
    assert!(matches!(exhausted, ScanOutcome::Stopped {
        completed_state: 3,
        completed_prefix,
        boundary: Some(identity),
        reason: MapStop::WorkExhausted { .. },
        ..
    } if completed_prefix == vec![1, 3] && identity == PartitionIdentity::new(3)));

    let token = CancellationToken::new();
    let lease = lease(10, token.clone());
    let cancelled = scan().run(Some(&lease), 0_u64, 0, 0, 0, 0, |carry, item, context| {
        context.checkpoint(1)?;
        if *item == 1 {
            token.cancel();
        }
        let next = *carry + *item;
        Ok::<_, MapKernelFailure<()>>((next, next))
    });
    assert!(matches!(cancelled, ScanOutcome::Stopped {
        completed_state: 1,
        completed_prefix,
        boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::Stop(MapKernelStop::Cancelled), .. },
        ..
    } if completed_prefix == vec![1] && identity == PartitionIdentity::new(2)));
}

#[test]
fn oversized_domain_error_cannot_escape_reserved_capacity() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = lease(10, CancellationToken::new());
    let outcome = scan().run(Some(&lease), 0_u64, 0, 0, 8, 0, |_carry, _, context| {
        context.checkpoint(1)?;
        Err::<(u64, u64), _>(MapKernelFailure::Domain("x".repeat(1_048_576)))
    });
    assert!(matches!(&outcome, ScanOutcome::Stopped {
        completed_state: 0,
        completed_prefix,
        boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::ResultCapacityExceeded, .. },
        ..
    } if completed_prefix.is_empty() && *identity == PartitionIdentity::new(1)));
    assert!(outcome.report().physical().peak_charged_memory_bytes() <= 4096);
}

#[test]
fn identity_storage_is_admitted_before_scan_dispatch() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let ids: Vec<_> = (0..100_000).map(PartitionIdentity::new).collect();
    let entries = ids.iter().copied().map(|identity| (identity, ())).collect();
    let scan = ExecutionScan::try_from_ordered(ids, entries).unwrap();
    let calls = AtomicUsize::new(0);
    let lease = lease(10, CancellationToken::new());
    let outcome = scan.run(Some(&lease), 0_u64, 0, 0, 0, 0, |state, _, _| {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok::<_, MapKernelFailure<()>>((*state, ()))
    });
    assert!(matches!(
        outcome,
        ScanOutcome::Stopped {
            reason: MapStop::Admission(LeaseDenial::ResourceExhausted),
            ..
        }
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}
