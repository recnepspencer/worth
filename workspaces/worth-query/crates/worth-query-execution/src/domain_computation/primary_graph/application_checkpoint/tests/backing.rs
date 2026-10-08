//! Physical byte custody across Query framing and native handoff. Literal native
//! bytes deliberately prove no Relational recovery admission or graph authority.
use std::sync::Arc;
use worth_relational::facade::durability::RelationalNativeCheckpoint;

use super::super::WorthQueryCheckpointCapturePolicy as CapturePolicy;
use super::{checkpoint_from_body, WorthQueryApplicationCheckpoint, FORMAT_VERSION};
use crate::domain_computation::primary_graph::tests::recoverable_commit_support::recoverable_application_world;

#[test]
fn untrusted_query_clones_share_moved_bytes_after_original_owner_drops() {
    let bytes = b"untrusted-query".to_vec().into_boxed_slice();
    let expected = bytes.as_ptr();
    let checkpoint = WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes);
    let clones = [checkpoint.clone(), checkpoint.clone(), checkpoint.clone()];
    drop(checkpoint);
    for clone in &clones {
        assert_eq!(clone.bytes(), b"untrusted-query");
        assert_eq!(clone.bytes().as_ptr(), expected);
        assert_eq!(clone.charged_payload_bytes(), None);
        assert_eq!(
            clone,
            &WorthQueryApplicationCheckpoint::from_untrusted_bytes(b"untrusted-query".to_vec())
        );
    }
}

#[test]
fn query_clones_decode_into_the_same_native_payload_backing() {
    let mut body = Vec::new();
    body.extend_from_slice(&FORMAT_VERSION.to_be_bytes());
    body.extend_from_slice(&1_u64.to_be_bytes());
    body.extend_from_slice(&3_u64.to_be_bytes());
    body.extend_from_slice(&0_u64.to_be_bytes());
    body.extend_from_slice(b"abc");
    let checkpoint = checkpoint_from_body(body);
    let bytes = Arc::new(checkpoint.bytes().to_vec().into_boxed_slice());
    let lifetime = Arc::downgrade(&bytes);
    drop(checkpoint);
    let checkpoint = WorthQueryApplicationCheckpoint {
        bytes: worth_execution::ExecutionImmutableBytes::from_external_bytes(bytes),
    };
    assert_three_handoffs_share_backing(checkpoint, b"abc", 1);
    assert!(lifetime.upgrade().is_none());
}

#[test]
fn encoded_query_clones_preserve_shared_backing_through_native_handoff() {
    let (world, _receipt) = recoverable_application_world(142, "checkpoint-backing-proof");
    let publication = world.application.publication();
    let native = b"literal-native-backing-payload";
    let (checkpoint, sections) = WorthQueryApplicationCheckpoint::encode(
        RelationalNativeCheckpoint::from_untrusted_bytes(native.to_vec()),
        publication,
        &[],
        CapturePolicy::SystemAllocation,
    )
    .expect("the actual production formatter creates an authenticated Query frame");
    assert_eq!(sections.total_bytes(), checkpoint.bytes().len());
    assert_three_handoffs_share_backing(checkpoint, native, publication.bootstrap_commit_id().0);
}

fn assert_three_handoffs_share_backing(
    checkpoint: WorthQueryApplicationCheckpoint,
    native_bytes: &[u8],
    bootstrap: u64,
) {
    let expected = checkpoint
        .bytes()
        .as_ptr()
        .wrapping_add(super::super::HEADER_BYTES);
    let clones = [checkpoint.clone(), checkpoint.clone(), checkpoint.clone()];
    for clone in &clones {
        assert_eq!(clone.bytes().as_ptr(), checkpoint.bytes().as_ptr());
    }
    drop(checkpoint);
    let decoded = clones.map(|clone| clone.decode().expect("Query framing is admitted"));
    for handoff in &decoded {
        assert_eq!(handoff.native.bytes(), native_bytes);
        assert_eq!(handoff.native.bytes().as_ptr(), expected);
        assert_eq!(handoff.native.captured_sections(), None);
        assert_eq!(handoff.bootstrap_commit_id.0, bootstrap);
        assert!(handoff.accepted_outputs.is_empty());
    }
    let native_clone = decoded[0].native.clone();
    drop(decoded);
    assert_eq!(native_clone.bytes(), native_bytes);
    assert_eq!(native_clone.bytes().as_ptr(), expected);
    drop(native_clone);
}
