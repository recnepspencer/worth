//! Command frames can outlive their returned capsule lease.
use super::*;

#[test]
fn command_observer_prevents_recycling_and_keeps_actual_charge_after_owner_disposal() {
    let fixture = fixture();
    let ledger = prepared(&fixture);
    let observer = capsule(&ledger);
    let mut lease = observer.take_command_buffer().unwrap();
    lease
        .buffer_mut()
        .bytes_mut()
        .unwrap()
        .extend_from_slice(b"settling-command");
    let frame = lease.buffer_mut().frame();
    assert!(lease.buffer_mut().bytes_mut().is_none());
    let retained = fixture.ports.counters().active_operation_bytes();
    drop(observer);
    drop(ledger);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), retained);
    drop(lease);
    assert!(fixture.ports.counters().active_operation_bytes() > 0);
    assert_eq!(frame.bytes(), b"settling-command");
    drop(frame);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 0);
}

#[test]
fn shared_returned_command_requires_prefunded_distinct_successor_before_drop() {
    let fixture = fixture();
    let mut ledger = prepared(&fixture);
    let observer = capsule(&ledger);
    let original_capsule = Arc::as_ptr(&observer);
    let mut lease = observer.take_command_buffer().unwrap();
    let bytes = lease.buffer_mut().bytes_mut().unwrap();
    let original_command = bytes.as_ptr();
    let original_capacity = bytes.capacity();
    bytes.extend_from_slice(b"retained-retry-command");
    let frame = lease.buffer_mut().frame();
    drop(observer);
    drop(lease);
    // Only the slot owns the capsule, but the returned inner buffer is shared.
    {
        let observer = capsule(&ledger);
        assert_eq!(Arc::strong_count(&observer), 2);
        assert!(!observer.returned_command_is_exclusive());
    }
    let original_backing_capacities = backing_capacities(&ledger);
    let retained = fixture.ports.counters().active_operation_bytes();
    let collision = fixture
        .ports
        .begin_operation(Scope::Recovery, NonZeroU64::new(65_535 - retained).unwrap())
        .unwrap();
    let result =
        ledger.prepare_checkpoint_backing(&fixture.owner, fixture.ceiling, Some(key()), false);
    let Err(super::super::super::ReleaseCertificateCapacityDenial::Resident(
        crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial::OperationAllocation(cause),
    )) = result
    else {
        panic!("shared command requires native successor admission before drop");
    };
    let pressure = cause.pressure().unwrap();
    assert!(pressure.requested() > 1);
    assert_eq!(pressure.admitted(), 65_535);
    assert_eq!(pressure.limit(), 65_536);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 65_535);
    assert_eq!(Arc::as_ptr(&capsule(&ledger)), original_capsule);
    assert_eq!(frame.bytes().as_ptr(), original_command);
    assert_eq!(frame.bytes(), b"retained-retry-command");
    assert_eq!(backing_capacities(&ledger), original_backing_capacities);
    drop(collision);
    ledger
        .prepare_checkpoint_backing(&fixture.owner, fixture.ceiling, Some(key()), false)
        .unwrap();
    let successor = capsule(&ledger);
    assert_ne!(Arc::as_ptr(&successor), original_capsule);
    assert!(successor.returned_command_is_exclusive());
    let mut successor_lease = successor.take_command_buffer().unwrap();
    let successor_bytes = successor_lease.buffer_mut().bytes_mut().unwrap();
    assert_ne!(successor_bytes.as_ptr(), original_command);
    assert_eq!(successor_bytes.capacity(), original_capacity);
    successor_bytes.extend_from_slice(b"distinct-successor-command");
    assert_eq!(frame.bytes(), b"retained-retry-command");
    assert_eq!(frame.bytes().as_ptr(), original_command);
    drop(successor_lease);
    drop(successor);
    drop(ledger);
    assert!(fixture.ports.counters().active_operation_bytes() > 0);
    assert_eq!(frame.bytes(), b"retained-retry-command");
    drop(frame);
    assert_eq!(fixture.ports.counters().active_operation_bytes(), 0);
}

fn backing_capacities(ledger: &SelectedReleaseCustodyLedger) -> [usize; 7] {
    let mut available = ledger
        .checkpoint_backing
        .as_ref()
        .unwrap()
        .available
        .lock()
        .unwrap();
    let capsule = Arc::get_mut(available.as_mut().unwrap()).unwrap();
    let preparation = capsule.take_preparation();
    let observer = &preparation.observer;
    let fold = preparation.fold.as_ref().unwrap();
    let capacities = [
        observer.bytes.capacity(),
        observer.records.capacity(),
        observer.scratch.capacity(),
        observer.frame_scratch.capacity(),
        fold.heads.owned_heap_bytes().unwrap() as usize,
        fold.batches.capacity(),
        fold.scratch.capacity(),
    ];
    capsule.put_preparation(preparation);
    capacities
}
