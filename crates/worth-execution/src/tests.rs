use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, OnceLock,
    },
    thread,
    time::{Duration, Instant},
};

use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

use crate::{
    authority::{
        CancellationSource, CancellationToken, ConstructionDenial, ExecutionAuthority,
        ExecutionAuthorityConfig, LeaseDenial, LeaseRequest, MemoryLimitDenial, MemoryLimitLevel,
        SlotRefusal,
    },
    backend::{
        run_checked_batch, AdmittedBatch, BackendKind, BatchStop, KernelFailure, KernelStop,
    },
};

static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
static TEST_LOCK: Mutex<()> = Mutex::new(());

mod adversarial;
mod controlled_child;
mod map_dispatch;
mod memory_level;
mod nesting;
mod owned_map;
mod prepared_map;
mod request_consultation;

fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(2_000),
        })
        .expect("one process authority")
    })
}

fn request(workers: usize, memory: u64, work: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(workers).unwrap(), memory, work),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

fn batch(values: &[u64], memory_each: u64) -> AdmittedBatch<u64> {
    let entries = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            (
                PartitionIdentity::new(index as u64 + 1),
                *value,
                memory_each,
                0,
            )
        })
        .collect();
    AdmittedBatch::try_admit(entries, 0).unwrap_or_else(|_| panic!("canonical fixture"))
}

#[test]
fn authority_is_process_singleton() {
    let _serial = TEST_LOCK.lock().unwrap();
    let authority = authority();
    assert_eq!(
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(1).unwrap(),
            charged_memory_bytes: Some(1),
        })
        .unwrap_err(),
        ConstructionDenial::AlreadyConstructed,
    );
    assert_eq!(
        authority.request_lease(request(5, 1, 1)).unwrap_err(),
        LeaseDenial::WorkerLimitExceedsParent
    );
    assert_eq!(
        authority.request_lease(request(1, 2_001, 1)).unwrap_err(),
        LeaseDenial::MemoryLimitExceedsParent
    );
}

#[test]
fn reservations_charge_every_ancestor_and_process_cap() {
    let _serial = TEST_LOCK.lock().unwrap();
    let authority = authority();
    let parent = authority.request_lease(request(3, 60, 10)).unwrap();
    assert_eq!(
        parent.child(request(4, 1, 1)).unwrap_err(),
        LeaseDenial::WorkerLimitExceedsParent
    );
    assert_eq!(
        parent.child(request(1, 61, 1)).unwrap_err(),
        LeaseDenial::MemoryLimitExceedsParent
    );
    let first = parent.child(request(2, 40, 10)).unwrap();
    let second = parent.child(request(2, 40, 10)).unwrap();
    let held = first.try_reserve(2, 35).unwrap();
    assert!(matches!(
        second.try_reserve(2, 1),
        Err(SlotRefusal::WorkersBusy)
    ));
    assert_eq!(
        second.try_reserve(1, 26).err(),
        Some(SlotRefusal::Denied(LeaseDenial::MemoryExhausted(
            MemoryLimitDenial {
                requested: 26,
                admitted: 25,
                level: MemoryLimitLevel::Policy { ancestor: 1 },
            }
        )))
    );
    assert!(matches!(
        first.try_reserve(1, 1),
        Err(SlotRefusal::WorkersBusy)
    ));
    let sibling = authority.request_lease(request(4, 100, 10)).unwrap();
    let process_held = sibling.try_reserve(2, 60).unwrap();
    assert!(matches!(
        sibling.try_reserve(1, 1),
        Err(SlotRefusal::WorkersBusy)
    ));
    drop(process_held);
    let memory_held = sibling.try_reserve(1, 100).unwrap();
    assert_eq!(
        sibling.try_reserve(1, 1).err(),
        Some(SlotRefusal::Denied(LeaseDenial::MemoryExhausted(
            MemoryLimitDenial {
                requested: 1,
                admitted: 0,
                level: MemoryLimitLevel::Policy { ancestor: 0 },
            }
        )))
    );
    drop(memory_held);
    drop(held);
    let recycled = second.try_reserve(2, 40).unwrap();
    drop(recycled);
}

#[test]
fn batch_memory_denial_precedes_kernel_dispatch() {
    let _serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(2, 10, 10)).unwrap();
    let admitted = batch(&[1, 2], 6);
    let called = AtomicBool::new(false);
    let outcome = run_checked_batch(
        Some(&lease),
        &admitted,
        BackendKind::Native,
        &|_, _| -> Result<u64, KernelFailure<()>> {
            called.store(true, Ordering::Release);
            Ok(1)
        },
    );
    assert!(!called.load(Ordering::Acquire));
    assert!(matches!(
        outcome.stop,
        Some(BatchStop::Admission(LeaseDenial::MemoryExhausted(
            MemoryLimitDenial { admitted: 10, .. }
        )))
    ));
    assert_eq!(outcome.report.charged_work(), 0);
}

#[test]
fn least_identity_failure_wins_after_later_partition_finishes() {
    let _serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(4, 1_000, 10)).unwrap();
    let later_finished = AtomicBool::new(false);
    let admitted = batch(&[1, 2], 1);
    let outcome = run_checked_batch(
        Some(&lease),
        &admitted,
        BackendKind::Native,
        &|value, context| -> Result<u64, KernelFailure<&'static str>> {
            context.checkpoint(1)?;
            if *value == 2 {
                later_finished.store(true, Ordering::Release);
                return Err(KernelFailure::Domain("later"));
            }
            let limit = Instant::now() + Duration::from_secs(2);
            while !later_finished.load(Ordering::Acquire) && Instant::now() < limit {
                thread::yield_now();
            }
            assert!(
                later_finished.load(Ordering::Acquire),
                "later task must finish first"
            );
            Err(KernelFailure::Domain("earlier"))
        },
    );
    assert_eq!(
        outcome.stop,
        Some(BatchStop::Failure {
            identity: PartitionIdentity::new(1),
            cause: KernelFailure::Domain("earlier"),
        })
    );
    assert_eq!(outcome.prefix_boundary, Some(PartitionIdentity::new(1)));
    assert!(outcome.values.is_empty());
    assert_eq!(outcome.report.physical().discarded_in_flight_work(), 1);
}

#[test]
fn serial_native_and_perturbed_schedules_settle_identically() {
    let _serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(4, 2_000, 100)).unwrap();
    let admitted = batch(&[2, 3, 5, 7, 11], 2);
    let kernel = |value: &u64, context: &mut crate::backend::KernelContext<'_, '_>| {
        context.checkpoint(*value)?;
        Ok::<_, KernelFailure<()>>(value * value)
    };
    let expected = run_checked_batch(Some(&lease), &admitted, BackendKind::Serial, &kernel);
    assert_eq!(expected.values, vec![4, 9, 25, 49, 121]);
    assert_eq!(expected.report.charged_work(), 28);
    assert_eq!(expected.report.charged_span(), 11);
    for backend in [
        BackendKind::Native,
        BackendKind::Perturbation(1),
        BackendKind::Perturbation(99),
    ] {
        let actual = run_checked_batch(Some(&lease), &admitted, backend, &kernel);
        assert_eq!(actual.values, expected.values);
        assert_eq!(actual.stop, expected.stop);
        assert_eq!(actual.prefix_boundary, expected.prefix_boundary);
        assert_eq!(actual.report.charged_work(), expected.report.charged_work());
        assert_eq!(actual.report.charged_span(), expected.report.charged_span());
    }
}

#[test]
fn work_ceiling_discards_later_results_at_canonical_boundary() {
    let _serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(4, 2_000, 5)).unwrap();
    let admitted = batch(&[2, 3, 4], 1);
    let result = run_checked_batch(
        Some(&lease),
        &admitted,
        BackendKind::Perturbation(7),
        &|value, context| {
            context.checkpoint(*value)?;
            Ok::<_, KernelFailure<()>>(*value)
        },
    );
    assert_eq!(result.values, vec![2, 3]);
    assert_eq!(
        result.stop,
        Some(BatchStop::WorkExhausted {
            identity: PartitionIdentity::new(3)
        })
    );
    assert_eq!(result.prefix_boundary, Some(PartitionIdentity::new(3)));
    assert_eq!(result.report.charged_work(), 5);
    assert_eq!(result.report.physical().discarded_in_flight_work(), 4);
}

#[test]
fn cancellation_and_deadline_stop_at_kernel_checkpoint() {
    let _serial = TEST_LOCK.lock().unwrap();
    let token = CancellationSource::new();
    let parent = authority()
        .request_lease(LeaseRequest {
            cancellation: token.token(),
            ..request(2, 1_000, 10)
        })
        .unwrap();
    let child = parent.child(request(1, 500, 10)).unwrap();
    token.cancel();
    let admitted = batch(&[1], 1);
    let kernel = |_: &u64, context: &mut crate::backend::KernelContext<'_, '_>| {
        context.checkpoint(1)?;
        Ok::<_, KernelFailure<()>>(1)
    };
    let cancelled = run_checked_batch(Some(&child), &admitted, BackendKind::Native, &kernel);
    assert_eq!(
        cancelled.stop,
        Some(BatchStop::Failure {
            identity: PartitionIdentity::new(1),
            cause: KernelFailure::Stop(KernelStop::Cancelled),
        })
    );
    assert_eq!(cancelled.report.charged_work(), 0);
    let expired = authority()
        .request_lease(LeaseRequest {
            deadline: Some(Instant::now() - Duration::from_secs(1)),
            ..request(1, 1_000, 10)
        })
        .unwrap();
    let timed_out = run_checked_batch(Some(&expired), &admitted, BackendKind::Native, &kernel);
    assert_eq!(
        timed_out.stop,
        Some(BatchStop::Failure {
            identity: PartitionIdentity::new(1),
            cause: KernelFailure::Stop(KernelStop::DeadlineElapsed),
        })
    );
    assert_eq!(timed_out.report.charged_work(), 0);
}

#[test]
fn panic_is_contained_and_next_batch_can_reuse_lease() {
    let _serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(2, 1_000, 10)).unwrap();
    let admitted = batch(&[1], 1);
    let failed = run_checked_batch(
        Some(&lease),
        &admitted,
        BackendKind::Native,
        &|_, _| -> Result<u64, KernelFailure<()>> { panic!("test kernel panic") },
    );
    assert_eq!(
        failed.stop,
        Some(BatchStop::Failure {
            identity: PartitionIdentity::new(1),
            cause: KernelFailure::Panic,
        })
    );
    let recovered = run_checked_batch(
        Some(&lease),
        &admitted,
        BackendKind::Native,
        &|value, context| {
            context.checkpoint(1)?;
            Ok::<_, KernelFailure<()>>(*value)
        },
    );
    assert_eq!(recovered.values, vec![1]);
    assert_eq!(recovered.stop, None);
}

#[test]
fn no_lease_uses_caller_thread_and_serial_posture() {
    let _serial = TEST_LOCK.lock().unwrap();
    let caller = thread::current().id();
    let admitted = batch(&[1, 2, 3], 1);
    let result = run_checked_batch(None, &admitted, BackendKind::Native, &|value, context| {
        assert_eq!(thread::current().id(), caller);
        context.checkpoint(1)?;
        Ok::<_, KernelFailure<()>>(*value)
    });
    assert_eq!(result.values, vec![1, 2, 3]);
    assert_eq!(result.report.resolved_posture(), ExecutionPosture::Serial);
    assert_eq!(result.report.physical().active_workers_high_watermark(), 0);
}
