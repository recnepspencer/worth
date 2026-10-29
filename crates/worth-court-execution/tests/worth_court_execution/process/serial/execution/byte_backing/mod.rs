// Actual sealed payload custody. Allocator/header/ledger metadata is uncharged.
#[allow(unused_imports)]
pub(crate) use {worth_execution::CancellationSource, worth_execution::CancellationToken, std::time::Duration, worth_execution::ExecutionByteBuffer, std::time::Instant, worth_execution::ExecutionAllocationDenialKind as Kind, worth_execution::MemoryLimitDenial, worth_execution::MemoryLimitLevel, worth_execution::ExecutionAllocationPolicy as Policy, crate::byte_backing::ordinary_context::SERIAL, crate::byte_backing::ordinary_context::authority, crate::byte_backing::ordinary_context::request, crate::test_serialization};

#[test]
fn fixed_backing_preserves_pointer_and_charge_through_shared_child_custody() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = crate::test_serialization::guard(&SERIAL);
    let parent = authority().request_lease(request(8)).unwrap();
    let retained = {
        let child = parent.child(request(4)).unwrap();
        let mut buffer = worth_execution::ExecutionByteBuffer::allocate(4, worth_execution::ExecutionAllocationPolicy::Execution(&child)).unwrap();
        assert_eq!(buffer.capacity(), 4);
        assert!(buffer.is_empty());
        buffer.extend_from_slice(b"data").unwrap();
        let pointer = buffer.bytes().as_ptr();
        buffer.overwrite(1, b"O").unwrap();
        assert_eq!(buffer.bytes(), b"dOta");
        let denied = buffer.extend_from_slice(b"x").unwrap_err();
        assert_eq!(denied.kind(), worth_execution::ExecutionAllocationDenialKind::WriteBeyondReserved);
        assert_eq!(denied.requested_payload_bytes(), Some(4));
        assert_eq!(
            buffer.overwrite(usize::MAX, b"x").unwrap_err().kind(),
            worth_execution::ExecutionAllocationDenialKind::WriteBeyondReserved
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
        Err(worth_execution::MemoryLimitDenial::Capacity {
            requested: 5,
            admitted: 4,
            level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
        })
    ));
    let mut replacement = worth_execution::ExecutionByteBuffer::allocate(4, worth_execution::ExecutionAllocationPolicy::Execution(&parent)).unwrap();
    replacement.extend_from_slice(b"next").unwrap();
    assert_ne!(replacement.bytes().as_ptr(), clone.bytes().as_ptr());
    let replacement = replacement.seal().unwrap();
    assert!(matches!(
        parent.reserve_memory(1),
        Err(worth_execution::MemoryLimitDenial::Capacity {
            requested: 1,
            admitted: 0,
            level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
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

    let _serial = crate::test_serialization::guard(&SERIAL);
    let parent = authority().request_lease(request(8)).unwrap();
    let mut stop_request = request(8);
    let stop = worth_execution::CancellationSource::new();
    stop_request.cancellation = stop.token();
    let child = parent.child(stop_request.clone()).unwrap();
    let policy = worth_execution::ExecutionAllocationPolicy::Execution(&child);
    let mut buffer = worth_execution::ExecutionByteBuffer::allocate(8, policy).unwrap();
    buffer.extend_from_slice(b"half").unwrap();
    stop.cancel();
    let early = policy.check_live().unwrap_err();
    assert_eq!(early.kind(), worth_execution::ExecutionAllocationDenialKind::Cancelled);
    assert_eq!(early.requested_payload_bytes(), None);
    let denied = buffer.check_live().unwrap_err();
    assert_eq!(denied.kind(), worth_execution::ExecutionAllocationDenialKind::Cancelled);
    assert_eq!(denied.requested_payload_bytes(), Some(8));
    assert_eq!(buffer.extend_from_slice(b"rest").unwrap_err(), denied);
    assert_eq!(buffer.overwrite(0, b"lost").unwrap_err(), denied);
    assert_eq!(buffer.bytes(), b"half");
    assert_eq!(buffer.seal().unwrap_err(), denied);
    assert_eq!(parent.reserve_memory(8).unwrap().bytes(), 8);
    let denied = worth_execution::ExecutionByteBuffer::allocate(8, policy).unwrap_err();
    assert_eq!(denied.kind(), worth_execution::ExecutionAllocationDenialKind::Cancelled);
    assert_eq!(denied.requested_payload_bytes(), Some(8));
    stop_request.cancellation = worth_execution::CancellationToken::new();
    stop_request.deadline = Some(std::time::Instant::now().checked_sub(std::time::Duration::from_secs(1)).unwrap());
    let expired = parent.child(stop_request).unwrap();
    let policy = worth_execution::ExecutionAllocationPolicy::Execution(&expired);
    assert_eq!(
        policy.check_live().unwrap_err().kind(),
        worth_execution::ExecutionAllocationDenialKind::DeadlineElapsed
    );
    let denied = worth_execution::ExecutionByteBuffer::allocate(8, policy).unwrap_err();
    assert_eq!(denied.kind(), worth_execution::ExecutionAllocationDenialKind::DeadlineElapsed);
    assert_eq!(denied.requested_payload_bytes(), Some(8));
}
