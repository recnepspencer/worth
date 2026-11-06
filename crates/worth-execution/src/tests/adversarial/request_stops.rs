//! Original request stops gate every member even without kernel checkpoints.
use super::*;

#[test]
fn cancellation_and_deadline_gate_kernels_without_explicit_checkpoints() {
    let _serial = TEST_LOCK.lock().unwrap();
    let time = crate::MonotonicTimeController::new().unwrap();
    let held = time.hold();
    let admitted = map(&[1, 2], 0);
    let called = AtomicUsize::new(0);
    let cancelled_token = CancellationSource::new();
    cancelled_token.cancel();
    let cancelled = authority()
        .request_lease(LeaseRequest {
            cancellation: cancelled_token.token(),
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
            deadline: Some(time.clock().deadline_at(held)),
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

    let token = CancellationSource::new();
    let between = authority()
        .request_lease(LeaseRequest {
            cancellation: token.token(),
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

    let deadline = held + Duration::from_secs(1);
    let expires_between = authority()
        .request_lease(LeaseRequest {
            deadline: Some(time.clock().deadline_at(deadline)),
            ..request(1, 1_000, 10)
        })
        .unwrap();
    let deadline_calls = AtomicUsize::new(0);
    let expires_between_outcome = admitted.run(Some(&expires_between), |value, _| {
        deadline_calls.fetch_add(1, Ordering::Relaxed);
        time.advance(Duration::from_secs(1)).unwrap();
        Ok::<_, MapKernelFailure<()>>(*value)
    });
    assert_eq!(deadline_calls.load(Ordering::Relaxed), 1);
    assert!(matches!(expires_between_outcome, MapOutcome::Stopped {
        completed_prefix, boundary: Some(identity),
        reason: MapStop::Failure { cause: MapKernelFailure::Stop(MapKernelStop::DeadlineElapsed), .. }, ..
    } if completed_prefix == vec![1] && identity == PartitionIdentity::new(2)));
}
