//! Immutable byte custody only. Literal bytes are not recovery admission proof.
use super::{NativeCheckpointSectionBytes, RelationalNativeCheckpoint};
use std::sync::Arc;
use worth_execution::ExecutionImmutableBytes;

#[test]
fn untrusted_native_clones_share_moved_bytes_after_the_original_drops() {
    let bytes = b"native".to_vec().into_boxed_slice();
    let expected = bytes.as_ptr();
    let checkpoint = RelationalNativeCheckpoint::from_untrusted_bytes(bytes);
    let clones = [checkpoint.clone(), checkpoint.clone(), checkpoint.clone()];
    drop(checkpoint);

    for clone in &clones {
        assert_eq!(clone.bytes(), b"native");
        assert_eq!(clone.bytes().as_ptr(), expected);
        assert_eq!(clone.captured_sections(), None);
    }
    drop(clones);
}

#[test]
fn embedded_native_payload_and_clones_retain_the_same_region_and_backing() {
    let bytes = Arc::new(b"prefixnative-suffix".to_vec().into_boxed_slice());
    let expected = bytes.as_ptr().wrapping_add(6);
    let lifetime = Arc::downgrade(&bytes);
    let checkpoint = RelationalNativeCheckpoint::from_untrusted_bytes_region(
        ExecutionImmutableBytes::from_external_bytes(bytes),
        6..12,
    )
    .expect("the bounded byte region is retained, not authenticated");
    let cloned = checkpoint.clone();
    assert_eq!(checkpoint.bytes(), b"native");
    assert_eq!(checkpoint.bytes().as_ptr(), expected);
    assert_eq!(cloned.region, 6..12);
    assert_eq!(checkpoint.bytes().as_ptr(), cloned.bytes().as_ptr());
    assert_eq!(checkpoint.bytes.charged_payload_bytes(), None);
    assert_eq!(
        cloned,
        RelationalNativeCheckpoint::from_untrusted_bytes(b"native".to_vec())
    );
    drop(checkpoint);
    assert_eq!(cloned.bytes(), b"native");
    assert_eq!(cloned.bytes().as_ptr(), expected);
    assert!(lifetime.upgrade().is_some());
    drop(cloned);
    assert!(lifetime.upgrade().is_none());
}

#[test]
fn embedded_native_payload_rejects_invalid_ranges_without_retaining_them() {
    let bytes = Arc::new(b"native".to_vec().into_boxed_slice());
    for range in [3..7, 7..7, 5..3, 0..usize::MAX] {
        assert!(RelationalNativeCheckpoint::from_untrusted_bytes_region(
            ExecutionImmutableBytes::from_external_bytes(Arc::clone(&bytes)),
            range
        )
        .is_err());
    }
    // Empty regions at the end have always been valid byte custody; native
    // recovery remains responsible for refusing invalid checkpoint contents.
    let empty = RelationalNativeCheckpoint::from_untrusted_bytes_region(
        ExecutionImmutableBytes::from_external_bytes(bytes),
        6..6,
    )
    .unwrap();
    assert_eq!(empty.bytes(), b"");
}

#[test]
fn captured_native_clones_preserve_sections_and_share_payload_custody() {
    let mut buffer = worth_execution::ExecutionByteBuffer::allocate(
        32,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    buffer.extend_from_slice(&[7; 32]).unwrap();
    let bytes = buffer.seal().unwrap();
    let expected = bytes.bytes().as_ptr();
    let sections = NativeCheckpointSectionBytes {
        total: 32,
        envelopes: 8,
        branch_roots: 4,
        branch_cells: 4,
        partition_mirror: 8,
        derived_indexes: 4,
        framing_and_metadata: 4,
    };
    let checkpoint = RelationalNativeCheckpoint::from_captured_bytes(bytes, sections);
    let cloned = checkpoint.clone();
    assert_eq!(checkpoint.bytes().as_ptr(), cloned.bytes().as_ptr());
    assert_eq!(cloned.region, checkpoint.region);
    assert_eq!(cloned.captured_sections(), Some(sections));
    // Capture metadata and allocation identity do not change byte-value equality.
    assert_eq!(
        cloned,
        RelationalNativeCheckpoint::from_untrusted_bytes(vec![7; 32])
    );
    drop(checkpoint);
    assert_eq!(cloned.bytes().as_ptr(), expected);
    assert_eq!(cloned.bytes(), &[7; 32]);
    drop(cloned);
}

#[test]
fn native_region_retains_the_whole_admitted_enclosing_backing_charge() {
    use std::num::NonZeroUsize;
    use worth_execution::{
        CancellationToken, ExecutionAllocationPolicy, ExecutionByteBuffer, LeaseDenial,
        LeaseRequest,
    };
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let payload = b"prefixnative-suffix";
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), payload.len() as u64, 1),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let mut builder =
        ExecutionByteBuffer::allocate(payload.len(), ExecutionAllocationPolicy::Execution(&lease))
            .unwrap();
    builder.extend_from_slice(payload).unwrap();
    let backing = builder.seal().unwrap();
    let expected = backing.bytes().as_ptr().wrapping_add(6);
    let checkpoint =
        RelationalNativeCheckpoint::from_untrusted_bytes_region(backing, 6..12).unwrap();
    let cloned = checkpoint.clone();
    drop(checkpoint);
    assert_eq!(cloned.bytes(), b"native");
    assert_eq!(cloned.bytes().as_ptr(), expected);
    assert_eq!(
        cloned.bytes.charged_payload_bytes(),
        Some(payload.len() as u64)
    );
    assert!(matches!(
        lease.reserve_memory(1),
        Err(LeaseDenial::ResourceExhausted)
    ));
    drop(cloned);
    assert_eq!(
        lease
            .reserve_memory(payload.len() as u64)
            .unwrap()
            .charged_bytes(),
        payload.len() as u64
    );
}
