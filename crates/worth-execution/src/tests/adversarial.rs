use std::sync::atomic::AtomicUsize;

use super::*;
use crate::backend::{run_scope, ScopeStop};
use crate::{ExecutionMap, MapKernelFailure, MapKernelStop, MapOutcome, MapPartition, MapStop};
use worth_foundational::ExecutionFallbackCause;

#[test]
fn zero_retained_reservation_survives_nested_scope_drop() {
    let _serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 1_000, 100)).unwrap();
    let outcome = map(&[1], 0).run(Some(&lease), |_, _context| {
        let mut zero = lease.reserve_retained_memory(0).unwrap();
        let nested = run_scope(Some(&lease), 0, 0, |_child| Ok::<_, KernelFailure<()>>(()));
        assert!(nested.result.is_ok());
        lease.rebind_retained_memory(&mut zero, 64).unwrap();
        drop(zero);
        Ok::<_, MapKernelFailure<()>>(0_u64)
    });
    assert!(matches!(outcome, MapOutcome::Complete { .. }));
}

fn map(values: &[u64], max_result_bytes: u64) -> ExecutionMap<u64, u64> {
    let expected = (0..values.len())
        .map(|index| PartitionIdentity::new(index as u64 + 1))
        .collect();
    let partitions = values
        .iter()
        .enumerate()
        .map(|(index, value)| MapPartition {
            identity: PartitionIdentity::new(index as u64 + 1),
            value: *value,
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes,
        })
        .collect();
    ExecutionMap::try_from_declared_partitions(expected, partitions)
        .unwrap_or_else(|denial| panic!("valid fixture: {denial:?}"))
}

#[test]
fn repeated_checkpoints_stop_at_local_work_ceiling() {
    let _serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 1_000, 1)).unwrap();
    let calls = AtomicUsize::new(0);
    let outcome = map(&[1], 0).run(Some(&lease), |_, context| {
        for _ in 0..1_000 {
            calls.fetch_add(1, Ordering::Relaxed);
            context.checkpoint(1)?;
        }
        Ok::<_, MapKernelFailure<()>>(1_u64)
    });
    assert_eq!(calls.load(Ordering::Relaxed), 2);
    match outcome {
        MapOutcome::Stopped {
            completed_prefix,
            boundary,
            reason,
            report,
        } => {
            assert!(completed_prefix.is_empty());
            assert_eq!(boundary, Some(PartitionIdentity::new(1)));
            assert_eq!(
                reason,
                MapStop::WorkExhausted {
                    identity: PartitionIdentity::new(1)
                }
            );
            assert_eq!(report.charged_work(), 1);
        }
        MapOutcome::Complete { .. } => panic!("work ceiling was bypassed"),
    }
}

#[test]
fn public_nested_map_charges_parent_work_and_span() {
    let _serial = TEST_LOCK.lock().unwrap();
    let parent = authority().request_lease(request(1, 1_000, 3)).unwrap();
    let child = parent.child(request(1, 500, 3)).unwrap();
    let outer = map(&[1], 0);
    let inner = map(&[2], 0);
    let outcome = outer.run(Some(&parent), |_, context| {
        context.checkpoint(1)?;
        let nested = inner.run(Some(&child), |value, context| {
            context.checkpoint(2)?;
            Ok::<_, MapKernelFailure<()>>(*value)
        });
        match nested {
            MapOutcome::Complete { values, report } => {
                assert_eq!(report.charged_work(), 2);
                Ok::<_, MapKernelFailure<()>>(values[0])
            }
            MapOutcome::Stopped { reason, .. } => panic!("unexpected nested stop: {reason:?}"),
        }
    });
    match outcome {
        MapOutcome::Complete { values, report } => {
            assert_eq!(values, vec![2]);
            assert_eq!(report.charged_work(), 3);
            assert_eq!(report.charged_span(), 3);
        }
        MapOutcome::Stopped { reason, .. } => panic!("unexpected parent stop: {reason:?}"),
    }
}

#[test]
fn public_nested_map_inherits_parent_remaining_work() {
    let _serial = TEST_LOCK.lock().unwrap();
    let parent = authority().request_lease(request(1, 1_000, 2)).unwrap();
    let child = parent.child(request(1, 500, 2)).unwrap();
    let outer = map(&[1], 0);
    let inner = map(&[2], 0);
    let outcome = outer.run(Some(&parent), |_, context| {
        context.checkpoint(1)?;
        let nested = inner.run(Some(&child), |_, context| {
            context.checkpoint(2)?;
            Ok::<_, MapKernelFailure<()>>(2_u64)
        });
        assert!(matches!(nested, MapOutcome::Stopped {
            reason: MapStop::WorkExhausted { identity }, ..
        } if identity == PartitionIdentity::new(1)));
        Ok::<_, MapKernelFailure<()>>(1_u64)
    });
    let report = outcome.report();
    assert!(matches!(outcome, MapOutcome::Stopped {
        boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::Stop(MapKernelStop::NestedStopped), .. },
        ..
    } if identity == PartitionIdentity::new(1)));
    assert_eq!(report.charged_work(), 1);
    assert_eq!(report.charged_span(), 1);
}

#[test]
fn cancellation_and_deadline_gate_kernels_without_explicit_checkpoints() {
    let _serial = TEST_LOCK.lock().unwrap();
    let admitted = map(&[1, 2], 0);
    let called = AtomicUsize::new(0);
    let cancelled_token = CancellationToken::new();
    cancelled_token.cancel();
    let cancelled = authority()
        .request_lease(LeaseRequest {
            cancellation: cancelled_token,
            ..request(1, 1_000, 10)
        })
        .unwrap();
    let cancelled_outcome = admitted.run(Some(&cancelled), |value, _| {
        called.fetch_add(1, Ordering::Relaxed);
        Ok::<_, MapKernelFailure<()>>(*value)
    });
    assert_eq!(called.load(Ordering::Relaxed), 0);
    assert!(matches!(cancelled_outcome, MapOutcome::Stopped {
        boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::Stop(MapKernelStop::Cancelled), .. },
        ..
    } if identity == PartitionIdentity::new(1)));

    let expired = authority()
        .request_lease(LeaseRequest {
            deadline: Some(Instant::now() - Duration::from_secs(1)),
            ..request(1, 1_000, 10)
        })
        .unwrap();
    let deadline_outcome = admitted.run(Some(&expired), |value, _| {
        called.fetch_add(1, Ordering::Relaxed);
        Ok::<_, MapKernelFailure<()>>(*value)
    });
    assert_eq!(called.load(Ordering::Relaxed), 0);
    assert!(matches!(deadline_outcome, MapOutcome::Stopped {
        boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::Stop(MapKernelStop::DeadlineElapsed), .. },
        ..
    } if identity == PartitionIdentity::new(1)));

    let token = CancellationToken::new();
    let between = authority()
        .request_lease(LeaseRequest {
            cancellation: token.clone(),
            ..request(1, 1_000, 10)
        })
        .unwrap();
    let between_outcome = admitted.run(Some(&between), |value, _| {
        called.fetch_add(1, Ordering::Relaxed);
        token.cancel();
        Ok::<_, MapKernelFailure<()>>(*value)
    });
    assert_eq!(called.load(Ordering::Relaxed), 1);
    assert!(matches!(between_outcome, MapOutcome::Stopped {
        completed_prefix, boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::Stop(MapKernelStop::Cancelled), .. }, ..
    } if completed_prefix == vec![1] && identity == PartitionIdentity::new(2)));

    let deadline = Instant::now() + Duration::from_millis(100);
    let expires_between = authority()
        .request_lease(LeaseRequest {
            deadline: Some(deadline),
            ..request(1, 1_000, 10)
        })
        .unwrap();
    let deadline_calls = AtomicUsize::new(0);
    let expires_between_outcome = admitted.run(Some(&expires_between), |value, _| {
        deadline_calls.fetch_add(1, Ordering::Relaxed);
        thread::sleep(deadline.saturating_duration_since(Instant::now()));
        while Instant::now() < deadline {
            thread::yield_now();
        }
        Ok::<_, MapKernelFailure<()>>(*value)
    });
    assert_eq!(deadline_calls.load(Ordering::Relaxed), 1);
    assert!(matches!(expires_between_outcome, MapOutcome::Stopped {
        completed_prefix, boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::Stop(MapKernelStop::DeadlineElapsed), .. }, ..
    } if completed_prefix == vec![1] && identity == PartitionIdentity::new(2)));
}

#[test]
fn framework_buffers_are_reserved_before_kernel_under_small_memory_caps() {
    let _serial = TEST_LOCK.lock().unwrap();
    let admitted = map(&[1], 0);
    let called = AtomicBool::new(false);
    for memory in [0, 1] {
        let lease = authority().request_lease(request(1, memory, 10)).unwrap();
        let outcome = admitted.run(Some(&lease), |value, _| {
            called.store(true, Ordering::Release);
            Ok::<_, MapKernelFailure<()>>(*value)
        });
        assert!(!called.load(Ordering::Acquire));
        assert!(matches!(
            outcome,
            MapOutcome::Stopped {
                reason: MapStop::Admission(LeaseDenial::ResourceExhausted),
                ..
            }
        ));
        assert_eq!(outcome.report().physical().peak_charged_memory_bytes(), 0);
    }
}

#[test]
fn oversized_result_is_rejected_at_its_partition_identity() {
    let _serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 1_000, 10)).unwrap();
    let admitted = map(&[1], 8);
    let outcome = admitted.run(Some(&lease), |_, context| {
        context.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>(String::with_capacity(16))
    });
    assert!(matches!(&outcome, MapOutcome::Stopped {
        completed_prefix, boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::ResultCapacityExceeded, .. }, ..
    } if completed_prefix.is_empty() && *identity == PartitionIdentity::new(1)));
    assert_eq!(outcome.report().charged_work(), 1);
}

#[test]
fn exhausted_process_slots_fall_back_to_one_worker_with_honest_report() {
    let _serial = TEST_LOCK.lock().unwrap();
    let authority = authority();
    let occupied = authority.request_lease(request(3, 100, 10)).unwrap();
    let three_slots = occupied.try_reserve(3, 0).unwrap();
    let lease = authority.request_lease(request(4, 1_000, 10)).unwrap();
    let outcome = map(&[1, 2], 0).run(Some(&lease), |value, context| {
        context.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>(*value)
    });
    let report = outcome.report();
    assert!(matches!(outcome, MapOutcome::Complete { values, .. } if values == vec![1, 2]));
    assert_eq!(report.resolved_posture(), ExecutionPosture::Serial);
    assert_eq!(report.fallback(), Some(ExecutionFallbackCause::Capacity));
    assert_eq!(report.physical().active_workers_high_watermark(), 1);
    assert_eq!(report.physical().peak_queue_width(), 2);
    assert!(report.physical().peak_charged_memory_bytes() > 0);
    drop(three_slots);
}

#[test]
fn enclosing_scope_enforces_one_ceiling_across_successive_nested_stages() {
    let _serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(2, 2_000, 1)).unwrap();
    let first = map(&[1], 0);
    let second = map(&[2], 0);
    let scoped = run_scope::<(), (), _>(Some(&lease), 0, 0, |_| {
        let first_result = first.run(Some(&lease), |value, context| {
            context.checkpoint(1)?;
            Ok::<_, MapKernelFailure<()>>(*value)
        });
        assert!(matches!(first_result, MapOutcome::Complete { values, .. } if values == vec![1]));
        let second_result = second.run(Some(&lease), |value, context| {
            context.checkpoint(1)?;
            Ok::<_, MapKernelFailure<()>>(*value)
        });
        assert!(matches!(second_result, MapOutcome::Stopped {
            reason: MapStop::WorkExhausted { identity }, ..
        } if identity == PartitionIdentity::new(1)));
        Ok(())
    });
    assert!(matches!(
        scoped.result,
        Err(ScopeStop::Failure(KernelFailure::Stop(
            KernelStop::NestedStopped
        )))
    ));
    assert_eq!(scoped.report.charged_work(), 1);
}

#[test]
fn unleased_scope_can_compose_serial_patterns_without_claiming_a_lease() {
    let _serial = TEST_LOCK.lock().unwrap();
    let inner = map(&[1], 0);
    let scoped = run_scope::<(), (), _>(None, 0, 0, |_| {
        let outcome = inner.run(None, |value, context| {
            context.checkpoint(2)?;
            Ok::<_, MapKernelFailure<()>>(*value)
        });
        assert!(matches!(outcome, MapOutcome::Complete { values, .. } if values == vec![1]));
        Ok(())
    });
    assert!(scoped.result.is_ok());
    assert_eq!(scoped.report.charged_work(), 2);
    assert_eq!(scoped.report.charged_span(), 2);
    assert_eq!(scoped.report.physical().active_workers_high_watermark(), 1);
}
