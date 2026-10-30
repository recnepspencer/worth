// Actual sealed payload custody. Allocator/header/ledger metadata is uncharged.
#[allow(unused_imports)]
use crate::byte_backing::ordinary_context::*;
#[allow(unused_imports)]
pub(crate) use crate::test_serialization;

#[test]
fn external_and_system_backings_are_uncharged_and_equal_by_bytes() {
    let bytes = std::sync::Arc::new(b"wire".to_vec().into_boxed_slice());
    let pointer = bytes.as_ptr();
    let lifetime = std::sync::Arc::downgrade(&bytes);
    let external = worth_execution::ExecutionImmutableBytes::from_external_bytes(bytes);
    assert_eq!(external.charged_payload_bytes(), None);
    let mut buffer = worth_execution::ExecutionByteBuffer::allocate(
        4,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
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
fn invalid_layout_and_incomplete_emission_never_seal_a_partial_payload() {
    let denied = worth_execution::ExecutionByteBuffer::allocate(
        usize::MAX,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap_err();
    assert_eq!(
        denied.classify(),
        worth_execution::ExecutionAllocationDenialClass::Physical(
            worth_execution::ExecutionPhysicalAllocationDenialKind::Layout
        )
    );
    assert_eq!(denied.requested_payload_bytes(), None);
    let mut buffer = worth_execution::ExecutionByteBuffer::allocate(
        3,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    buffer.extend_from_slice(b"a").unwrap();
    let denied = buffer.seal().unwrap_err();
    assert_eq!(
        denied.classify(),
        worth_execution::ExecutionAllocationDenialClass::Physical(
            worth_execution::ExecutionPhysicalAllocationDenialKind::IncompleteSeal
        )
    );
    assert_eq!(denied.requested_payload_bytes(), Some(3));
}

#[test]
fn chunked_byte_authoring_keeps_pointer_and_overwrites_exact_written_range() {
    let mut builder = worth_execution::ExecutionByteBuffer::allocate(
        2 * 64 * 1024 + 5,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
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
