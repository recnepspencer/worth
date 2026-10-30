//! Genuine Store activation and selected C8 checkpoint ancestry. Serving
//! remains sealed until Store's same-media tier rejoin is installed.

pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {
    std::path::Path, worth_proof::TransitionOutcome,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
    worth_store_testkit_recovery_runtime::production_entry::initialized_recovery_world,
    worth_store_testkit_recovery_runtime::production_entry::run_entry,
};
pub(crate) use {
    worth_store_physical_format::checkpoint_certificate_frame_bytes,
    worth_store_physical_format::decode_checkpoint_certificate,
    worth_store_physical_format::CheckpointCertificateKind,
    worth_store_physical_format::CheckpointStreamFooter,
    worth_store_physical_format::ReleaseCheckpointCertificateV1,
    worth_store_physical_format::ReleaseCheckpointNoReleaseV1,
    worth_store_physical_format::CHECKPOINT_CERTIFICATE_PREFIX_BYTES,
    worth_store_physical_format::CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES,
};

pub(crate) fn selected_no_release(root: &Path) -> ReleaseCheckpointNoReleaseV1 {
    let bytes =
        std::fs::read(root.join("families/checkpoint.current")).expect("selected checkpoint bytes");
    let footer_start = bytes.len() - CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES;
    let footer =
        worth_store_physical_format::CheckpointStreamFooter::decode_record(&bytes[footer_start..])
            .expect("selected certified footer");
    let mut offset = footer_start - footer.certificate_record_bytes() as usize;
    let mut marker = None;
    for _ in 0..footer.certificate_record_count() {
        let length = checkpoint_certificate_frame_bytes(
            &bytes[offset..offset + CHECKPOINT_CERTIFICATE_PREFIX_BYTES],
        )
        .expect("selected certificate length");
        let (kind, payload) = decode_checkpoint_certificate(&bytes[offset..offset + length])
            .expect("selected certificate frame");
        if kind == worth_store_physical_format::CheckpointCertificateKind::ReleasedDrop {
            let worth_store_physical_format::ReleaseCheckpointCertificateV1::NoRelease(value) =
                worth_store_physical_format::ReleaseCheckpointCertificateV1::decode(payload)
                    .expect("typed NoRelease marker")
            else {
                panic!("tier-only checkpoint must not carry release custody")
            };
            assert!(marker.replace(value).is_none());
        }
        offset += length;
    }
    assert_eq!(offset, footer_start);
    marker.expect("positive NoRelease selected marker")
}
