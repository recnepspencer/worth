//! Actual sealed payload custody. Allocator/header/ledger metadata is uncharged.
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAllocationDenialKind as Kind,
    ExecutionAllocationPolicy as Policy, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionByteBuffer, ExecutionImmutableBytes, LeaseRequest, MemoryLimitDenial,
    MemoryLimitLevel,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
static SERIAL: Mutex<()> = Mutex::new(());
fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(1).unwrap(),
            charged_memory_bytes: None,
        })
        .unwrap()
    })
}
fn request(bytes: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), bytes, 1),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

#[test]
fn fixed_backing_preserves_pointer_and_charge_through_shared_child_custody() {
    let _serial = SERIAL.lock().unwrap();
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
        Err(MemoryLimitDenial {
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
        Err(MemoryLimitDenial {
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
fn external_and_system_backings_are_uncharged_and_equal_by_bytes() {
    let bytes = Arc::new(b"wire".to_vec().into_boxed_slice());
    let pointer = bytes.as_ptr();
    let lifetime = Arc::downgrade(&bytes);
    let external = ExecutionImmutableBytes::from_external_bytes(bytes);
    assert_eq!(external.charged_payload_bytes(), None);
    let mut buffer = ExecutionByteBuffer::allocate(4, Policy::SystemAllocation).unwrap();
    buffer.extend_from_slice(b"wire").unwrap();
    let system = buffer.seal().unwrap();
    assert_eq!(system.charged_payload_bytes(), None);
    assert_eq!(system, external);
    let clone = external.clone();
    drop(external);
    assert_eq!(clone.bytes().as_ptr(), pointer);
    assert!(lifetime.upgrade().is_some());
    drop(clone);
    assert!(lifetime.upgrade().is_none());
    // Sharing grants no mutable byte view and Debug states no custody identity.
    assert!(!format!("{system:?}").contains("wire"));
}

#[test]
fn stopped_builder_retains_checked_quote_and_releases_actual_backing_charge() {
    let _serial = SERIAL.lock().unwrap();
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

#[test]
fn invalid_layout_and_incomplete_emission_never_seal_a_partial_payload() {
    let denied = ExecutionByteBuffer::allocate(usize::MAX, Policy::SystemAllocation).unwrap_err();
    assert_eq!(denied.kind(), Kind::Layout);
    assert_eq!(denied.requested_payload_bytes(), None);
    let mut buffer = ExecutionByteBuffer::allocate(3, Policy::SystemAllocation).unwrap();
    buffer.extend_from_slice(b"a").unwrap();
    let denied = buffer.seal().unwrap_err();
    assert_eq!(denied.kind(), Kind::IncompleteSeal);
    assert_eq!(denied.requested_payload_bytes(), Some(3));
}

#[test]
fn chunked_byte_authoring_keeps_pointer_and_overwrites_exact_written_range() {
    let mut builder =
        ExecutionByteBuffer::allocate(2 * 64 * 1024 + 5, Policy::SystemAllocation).unwrap();
    let prefix = vec![0x31; 64 * 1024 + 3];
    let suffix = vec![0x72; 64 * 1024 + 2];
    builder.extend_from_slice(&prefix).unwrap();
    let pointer = builder.bytes().as_ptr();
    builder.extend_from_slice(&suffix).unwrap();
    assert_eq!(builder.bytes().as_ptr(), pointer);
    let checksum_region = vec![0xa5; 64 * 1024 + 1];
    builder.overwrite(2, &checksum_region).unwrap();
    let sealed = builder.seal().unwrap();
    assert_eq!(sealed.bytes().as_ptr(), pointer);
    assert_eq!(&sealed[..2], &[0x31; 2]);
    assert_eq!(&sealed[2..64 * 1024 + 3], checksum_region.as_slice());
    assert!(sealed[64 * 1024 + 3..].iter().all(|byte| *byte == 0x72));
}
