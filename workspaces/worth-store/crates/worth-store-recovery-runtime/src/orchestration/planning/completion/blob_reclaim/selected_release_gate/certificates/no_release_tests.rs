use std::num::NonZeroU64;

use worth_store_physical_format::{
    encode_checkpoint_certificate,
    store_namespace::{ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion},
    CheckpointCertificateKind, PhysicalCheckpointIdentity, ReleaseCheckpointCertificateV1,
    ReleaseCheckpointNoReleaseV1,
};

use super::parse_records;

#[test]
fn selected_no_release_marker_is_positive_and_exclusive() {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([1; 16]).unwrap(),
    )
    .published_identity();
    let identity = PhysicalCheckpointIdentity::new(store, NonZeroU64::new(3).unwrap());
    let marker =
        ReleaseCheckpointNoReleaseV1::new(identity, 12, [3; 32], 2, [2; 32], [4; 32]).unwrap();
    let frame = encode_checkpoint_certificate(
        CheckpointCertificateKind::ReleasedDrop,
        &ReleaseCheckpointCertificateV1::NoRelease(marker).encode(),
    )
    .unwrap()
    .into_boxed_slice();
    assert!(parse_records(&[frame.clone()], identity, 12, [3; 32])
        .unwrap()
        .is_none());
    assert!(parse_records(&[], identity, 12, [3; 32]).unwrap().is_none());
    assert!(parse_records(&[frame.clone()], identity, 12, [9; 32]).is_err());
    assert!(parse_records(&[frame.clone(), frame], identity, 12, [3; 32]).is_err());
}
