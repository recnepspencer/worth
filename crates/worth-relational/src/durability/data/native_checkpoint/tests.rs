//! Immutable byte custody only. Literal bytes are not recovery admission proof.
use super::{NativeCheckpointSectionBytes, RelationalNativeCheckpoint};
use std::sync::Arc;

#[test]
fn untrusted_native_clones_share_moved_bytes_until_the_last_owner_drops() {
    let bytes = b"native".to_vec().into_boxed_slice();
    let expected = bytes.as_ptr();
    let checkpoint = RelationalNativeCheckpoint::from_untrusted_bytes(bytes);
    let lifetime = Arc::downgrade(&checkpoint.bytes);
    let clones = [checkpoint.clone(), checkpoint.clone(), checkpoint.clone()];
    drop(checkpoint);

    for clone in &clones {
        assert_eq!(clone.bytes(), b"native");
        assert_eq!(clone.bytes().as_ptr(), expected);
        assert_eq!(clone.captured_sections(), None);
    }
    assert!(lifetime.upgrade().is_some());
    drop(clones);
    assert!(lifetime.upgrade().is_none());
}

#[test]
fn embedded_native_payload_and_clones_retain_the_same_region_and_backing() {
    let bytes = Arc::new(b"prefixnative-suffix".to_vec().into_boxed_slice());
    let expected = bytes.as_ptr().wrapping_add(6);
    let lifetime = Arc::downgrade(&bytes);
    let checkpoint = RelationalNativeCheckpoint::from_untrusted_bytes_region(bytes, 6..12)
        .expect("the bounded byte region is retained, not authenticated");
    let cloned = checkpoint.clone();
    assert_eq!(checkpoint.bytes(), b"native");
    assert_eq!(checkpoint.bytes().as_ptr(), expected);
    assert_eq!(cloned.region, 6..12);
    assert!(Arc::ptr_eq(&checkpoint.bytes, &cloned.bytes));
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
        assert!(
            RelationalNativeCheckpoint::from_untrusted_bytes_region(Arc::clone(&bytes), range)
                .is_err()
        );
    }
    // Empty regions at the end have always been valid byte custody; native
    // recovery remains responsible for refusing invalid checkpoint contents.
    let empty = RelationalNativeCheckpoint::from_untrusted_bytes_region(bytes, 6..6).unwrap();
    assert_eq!(empty.bytes(), b"");
}

#[test]
fn captured_native_clones_preserve_sections_and_share_payload_custody() {
    let bytes = vec![7; 32];
    let expected = bytes.as_ptr();
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
    let lifetime = Arc::downgrade(&checkpoint.bytes);
    let cloned = checkpoint.clone();
    assert!(Arc::ptr_eq(&checkpoint.bytes, &cloned.bytes));
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
    assert!(lifetime.upgrade().is_some());
    drop(cloned);
    assert!(lifetime.upgrade().is_none());
}
