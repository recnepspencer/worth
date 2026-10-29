// Actual sealed payload custody. Allocator/header/ledger metadata is uncharged.
#[allow(unused_imports)]
pub(crate) use crate::fast::byte_backing::*;

#[test]
fn fixed_backing_preserves_pointer_and_charge_through_shared_child_custody() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = test_serialization::guard(&SERIAL);
    let parent = authority().request_lease(request(8)).unwrap();
    let retained = {
        let child = parent.child(request(4)).unwrap();
        let mut buffer = ExecutionByteBuffer::allocate(4, Policy::Execution(&child)).unwrap();
        assert_eq!(buffer.capacity(), 4);
        assert!(buffer.is_empty());
        buffer.extend_from_slice(b"data").unwrap();
        let pointer = buffer.bytes().as_ptr();
        buffer.overwrite(1, b"O").unwrap();
        assert_eq!(buffer.bytes(), b"dOta");
        let denied = buffer.extend_from_slice(b"x").unwrap_err();
        assert_eq!(denied.kind(), Kind::WriteBeyondReserved);
        assert_eq!(denied.requested_payload_bytes(), Some(4));
        assert_eq!(
            buffer.overwrite(usize::MAX, b"x").unwrap_err().kind(),
            Kind::WriteBeyondReserved
        );
        assert_eq!(buffer.bytes(), b"dOta");
        assert_eq!(buffer.bytes().as_ptr(), pointer);
        let sealed = buffer.seal().unwrap();
        assert_eq!(sealed.bytes().as_ptr(), pointer);
        assert_eq!(sealed.charged_payload_bytes(), Some(4));
        sealed
    };
    // The child lease is gone; the shared allocation still charges its parent.
    assert_eq!(retained.as_ref(), b"dOta");
    let clone = retained.clone();
    drop(retained);
    assert!(matches!(
        parent.reserve_memory(5),
        Err(MemoryLimitDenial::Capacity {
            requested: 5,
            admitted: 4,
            level: MemoryLimitLevel::Policy { ancestor: 0 }
        })
    ));
    let mut replacement = ExecutionByteBuffer::allocate(4, Policy::Execution(&parent)).unwrap();
    replacement.extend_from_slice(b"next").unwrap();
    assert_ne!(replacement.bytes().as_ptr(), clone.bytes().as_ptr());
    let replacement = replacement.seal().unwrap();
    assert!(matches!(
        parent.reserve_memory(1),
        Err(MemoryLimitDenial::Capacity {
            requested: 1,
            admitted: 0,
            level: MemoryLimitLevel::Policy { ancestor: 0 }
        })
    ));
    drop(clone);
    assert_eq!(parent.reserve_memory(4).unwrap().bytes(), 4);
    drop(replacement);
    assert_eq!(parent.reserve_memory(8).unwrap().bytes(), 8);
}

#[test]
fn stopped_builder_retains_checked_quote_and_releases_actual_backing_charge() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = test_serialization::guard(&SERIAL);
    let parent = authority().request_lease(request(8)).unwrap();
    let mut stop_request = request(8);
    let stop = CancellationSource::new();
    stop_request.cancellation = stop.token();
    let child = parent.child(stop_request.clone()).unwrap();
    let policy = Policy::Execution(&child);
    let mut buffer = ExecutionByteBuffer::allocate(8, policy).unwrap();
    buffer.extend_from_slice(b"half").unwrap();
    stop.cancel();
    let early = policy.check_live().unwrap_err();
    assert_eq!(early.kind(), Kind::Cancelled);
    assert_eq!(early.requested_payload_bytes(), None);
    let denied = buffer.check_live().unwrap_err();
    assert_eq!(denied.kind(), Kind::Cancelled);
    assert_eq!(denied.requested_payload_bytes(), Some(8));
    assert_eq!(buffer.extend_from_slice(b"rest").unwrap_err(), denied);
    assert_eq!(buffer.overwrite(0, b"lost").unwrap_err(), denied);
    assert_eq!(buffer.bytes(), b"half");
    assert_eq!(buffer.seal().unwrap_err(), denied);
    assert_eq!(parent.reserve_memory(8).unwrap().bytes(), 8);
    let denied = ExecutionByteBuffer::allocate(8, policy).unwrap_err();
    assert_eq!(denied.kind(), Kind::Cancelled);
    assert_eq!(denied.requested_payload_bytes(), Some(8));
    stop_request.cancellation = CancellationToken::new();
    stop_request.deadline = Some(Instant::now().checked_sub(Duration::from_secs(1)).unwrap());
    let expired = parent.child(stop_request).unwrap();
    let policy = Policy::Execution(&expired);
    assert_eq!(
        policy.check_live().unwrap_err().kind(),
        Kind::DeadlineElapsed
    );
    let denied = ExecutionByteBuffer::allocate(8, policy).unwrap_err();
    assert_eq!(denied.kind(), Kind::DeadlineElapsed);
    assert_eq!(denied.requested_payload_bytes(), Some(8));
}
