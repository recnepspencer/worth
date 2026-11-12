use super::*;
use crate::physical_residency::tests::{
    allocation, coordinate, expect_fault, limits, store, READ_SCOPE,
};
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

#[test]
fn original_pin_and_operation_retire_once_while_inert_pool_custody_survives_close() {
    let pool = PhysicalResidencyPool::open(store(151), limits(64, 2, 1, 64, 3)).unwrap();
    let grant = allocation(&pool, READ_SCOPE);
    let key = PhysicalFrameKey::new(store(151), coordinate(1, 8));
    let pin = expect_fault(&pool, &grant, key)
        .load(|bytes| {
            bytes.fill(19);
            Ok::<_, ()>(())
        })
        .unwrap();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 20)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let original_bytes = Arc::as_ptr(&pin.bytes);
    let original_generation = pin.resident_generation();

    let gate = pool.inner.state.lock().unwrap();
    let (pin, cause) = pin.try_retire_for_request(&request).unwrap_err();
    assert_eq!(cause, Denial::Busy);
    assert_eq!(Arc::as_ptr(&pin.bytes), original_bytes);
    assert_eq!(pin.resident_generation(), original_generation);
    assert_eq!(&*pin, &[19; 8]);
    drop(gate);
    let before = pool.counters();
    let retired_pin = pin.try_retire_for_request(&request).unwrap();
    assert_eq!(pool.counters().pin_leases(), 0);
    assert_eq!(pool.counters().resident_bytes(), before.resident_bytes());
    assert_eq!(
        pool.counters().administrative_drains(),
        before.administrative_drains()
    );

    let gate = pool.inner.state.lock().unwrap();
    let (grant, cause) = grant.try_retire_for_request(&request).unwrap_err();
    assert_eq!(cause, Denial::Busy);
    assert_eq!(grant.bytes(), 1);
    drop(gate);
    let retired_grant = grant.try_retire_for_request(&request).unwrap();
    assert_eq!(pool.counters().active_operation_bytes(), 0);
    assert!(!pool.close().requires_inspection());
    let original_pool = Arc::downgrade(&pool.inner);
    drop(pool);
    assert!(
        original_pool.upgrade().is_some(),
        "actual inert owners retain their original allocation"
    );
    drop(retired_pin);
    drop(retired_grant);
    assert!(original_pool.upgrade().is_none());
    assert_eq!(serial.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn stopped_and_closed_owners_refuse_before_any_pin_release_or_drain() {
    let pool = PhysicalResidencyPool::open(store(152), limits(64, 2, 1, 64, 3)).unwrap();
    let grant = allocation(&pool, READ_SCOPE);
    let pin = expect_fault(
        &pool,
        &grant,
        PhysicalFrameKey::new(store(152), coordinate(1, 8)),
    )
    .load(|bytes| {
        bytes.fill(7);
        Ok::<_, ()>(())
    })
    .unwrap();
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 20)),
        cancellation.token(),
        None,
    );
    cancellation.cancel();
    let before = pool.counters();
    let (pin, cause) = pin
        .try_retire_for_request(&ExecutionRequest::serial(&serial))
        .unwrap_err();
    assert!(matches!(cause, Denial::Execution(_)));
    assert_eq!(pool.counters(), before);
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 20)),
        CancellationSource::new().token(),
        None,
    );
    assert!(pool.close().requires_inspection());
    let before = pool.counters();
    let (pin, cause) = pin
        .try_retire_for_request(&ExecutionRequest::serial(&serial))
        .unwrap_err();
    assert_eq!(cause, Denial::PoolClosed);
    assert_eq!(pool.counters(), before);
    assert_eq!(&*pin, &[7; 8]);
    // Explicit ordinary abandonment is outside the bounded release claim.
    drop(pin);
    drop(grant);
}
