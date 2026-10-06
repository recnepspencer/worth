use super::adversarial::map;
use super::*;
use crate::backend::{enter_certification_activity, run_scope, ScopeStop};
use crate::{MapKernelFailure, MapKernelStop, MapOutcome, MapStop};

#[test]
fn prepared_map_reserves_live_memory_before_dispatch_and_reports_it_once() {
    let _serial = TEST_LOCK.lock().unwrap();
    let authority = authority();
    let parent = authority.request_lease(request(1, 1_000, 10)).unwrap();
    let child = parent.child(request(1, 1_000, 10)).unwrap();
    let occupied = authority.request_lease(request(1, 1_999, 10)).unwrap();
    let held = occupied.try_reserve(0, 1_999).unwrap();
    assert!(matches!(
        map(&[7], 0).prepare_run::<u64, ()>(child),
        Err(LeaseDenial::MemoryExhausted(MemoryLimitDenial {
            admitted: 1,
            ..
        }))
    ));
    drop(held);
    let child = parent.child(request(1, 1_000, 10)).unwrap();
    let prepared = map(&[7], 0)
        .prepare_run::<u64, ()>(child)
        .unwrap_or_else(|denial| panic!("prepared map denied: {denial:?}"));
    assert!(matches!(
        occupied.try_reserve(0, 1_999),
        Err(SlotRefusal::Denied(LeaseDenial::MemoryExhausted(
            MemoryLimitDenial {
                requested: 1_999,
                ..
            }
        )))
    ));
    let outcome = prepared.run(|value, work| {
        work.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>(*value)
    });
    assert!(matches!(&outcome, MapOutcome::Complete { values, .. } if values == &vec![7]));
    assert_eq!(outcome.report().charged_work(), 1);
    assert!(outcome.report().physical().peak_charged_memory_bytes() > 0);
    let released = occupied.try_reserve(0, 1_999).unwrap();
    drop(released);
}

#[test]
fn prepared_map_cannot_dispatch_under_a_replaced_parent_context() {
    let _serial = TEST_LOCK.lock().unwrap();
    let parent = authority().request_lease(request(1, 1_500, 10)).unwrap();
    let mut prepared = None;
    let scope = run_scope(Some(&parent), 0, 0, |_| {
        let child = parent.child(request(1, 1_500, 10)).unwrap();
        prepared = Some(
            map(&[3], 0)
                .prepare_run::<u64, ()>(child)
                .unwrap_or_else(|denial| panic!("nested preparation denied: {denial:?}")),
        );
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(scope.result.is_ok());
    let called = AtomicBool::new(false);
    let outcome = prepared.expect("scope prepared one map").run(|_, _| {
        called.store(true, Ordering::SeqCst);
        Ok::<_, MapKernelFailure<()>>(3_u64)
    });
    assert!(!called.load(Ordering::SeqCst));
    assert!(matches!(
        outcome,
        MapOutcome::Stopped {
            reason: MapStop::Admission(LeaseDenial::UnrelatedNestedLease),
            ..
        }
    ));
}

#[test]
fn prepared_map_cannot_dispatch_under_a_replaced_certification_ledger() {
    let _serial = TEST_LOCK.lock().unwrap();
    let parent = authority().request_lease(request(1, 1_500, 10)).unwrap();
    let prepared = {
        let _first_ledger = enter_certification_activity();
        let child = parent.child(request(1, 1_500, 10)).unwrap();
        map(&[3], 0)
            .prepare_run::<u64, ()>(child)
            .unwrap_or_else(|denial| panic!("certified preparation denied: {denial:?}"))
    };
    let _different_ledger = enter_certification_activity();
    let called = AtomicBool::new(false);
    let outcome = prepared.run(|_, _| {
        called.store(true, Ordering::SeqCst);
        Ok::<_, MapKernelFailure<()>>(3_u64)
    });
    assert!(!called.load(Ordering::SeqCst));
    assert!(matches!(
        outcome,
        MapOutcome::Stopped {
            reason: MapStop::Admission(LeaseDenial::UnrelatedNestedLease),
            ..
        }
    ));
}

#[test]
fn prepared_map_uses_dispatch_time_work_and_cancellation() {
    let _serial = TEST_LOCK.lock().unwrap();
    let parent = authority().request_lease(request(1, 1_500, 2)).unwrap();
    let completed = AtomicBool::new(false);
    let scope = run_scope(Some(&parent), 0, 0, |work| {
        let child = parent.child(request(1, 1_500, 2)).unwrap();
        let prepared = map(&[5], 0)
            .prepare_run::<u64, ()>(child)
            .unwrap_or_else(|denial| panic!("nested preparation denied: {denial:?}"));
        work.checkpoint(2)?;
        let outcome = prepared.run(|_, work| {
            completed.store(true, Ordering::SeqCst);
            work.checkpoint(1)?;
            Ok::<_, MapKernelFailure<()>>(5_u64)
        });
        assert!(matches!(
            outcome,
            MapOutcome::Stopped {
                reason: MapStop::WorkExhausted { .. },
                ..
            }
        ));
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(matches!(scope.result, Err(ScopeStop::Failure(_))));
    assert!(completed.load(Ordering::SeqCst));

    let cancellation = CancellationSource::new();
    let admission = LeaseRequest {
        cancellation: cancellation.token(),
        ..request(1, 1_500, 10)
    };
    let lease = authority().request_lease(admission).unwrap();
    let child = lease.child(request(1, 1_500, 10)).unwrap();
    let prepared = map(&[9], 0)
        .prepare_run::<u64, ()>(child)
        .unwrap_or_else(|denial| panic!("preparation denied: {denial:?}"));
    let calls = AtomicBool::new(false);
    cancellation.cancel();
    let outcome = prepared.run(|_, _| {
        calls.store(true, Ordering::SeqCst);
        Ok::<_, MapKernelFailure<()>>(9_u64)
    });
    assert!(!calls.load(Ordering::SeqCst));
    assert!(matches!(
        outcome,
        MapOutcome::Stopped {
            reason: MapStop::Failure {
                cause: MapKernelFailure::Stop(MapKernelStop::Cancelled),
                ..
            },
            ..
        }
    ));
}

#[test]
fn prepared_empty_map_reports_its_retained_framework_memory() {
    let _serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 1_000, 10)).unwrap();
    let prepared = map(&[], 0)
        .prepare_run::<u64, ()>(lease)
        .unwrap_or_else(|denial| panic!("empty preparation denied: {denial:?}"));
    let outcome = prepared.run(|_, _| -> Result<u64, MapKernelFailure<()>> {
        panic!("empty map dispatched a callback")
    });
    assert!(matches!(&outcome, MapOutcome::Complete { values, .. } if values.is_empty()));
    assert!(outcome.report().physical().peak_charged_memory_bytes() > 0);
}
