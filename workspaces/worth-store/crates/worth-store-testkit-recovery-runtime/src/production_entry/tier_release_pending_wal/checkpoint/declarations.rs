pub use {std::fs, std::path::Path};
pub use {
    worth_store_physical_format::checkpoint_certificate_frame_bytes,
    worth_store_physical_format::decode_checkpoint_certificate,
    worth_store_physical_format::CheckpointCertificateKind,
    worth_store_physical_format::CheckpointStreamFooter,
    worth_store_physical_format::ReleaseCheckpointCertificateV1,
    worth_store_physical_format::TierEpochCheckpointCertificateV1,
    worth_store_physical_format::CHECKPOINT_CERTIFICATE_PREFIX_BYTES,
    worth_store_physical_format::CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES,
};

pub fn selected_checkpoint(root: &Path) -> Vec<u8> {
    std::fs::read(root.join("families/checkpoint.current")).expect("selected checkpoint")
}

pub fn assert_tier_and_no_release(bytes: &[u8]) {
    assert_tier_release_roster(bytes, false);
}

pub fn assert_tier_and_batch(bytes: &[u8]) {
    assert_tier_release_roster(bytes, true);
}

pub fn assert_tier_release_roster(bytes: &[u8], folded_batch: bool) {
    let footer_start = bytes.len() - CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES;
    let footer =
        worth_store_physical_format::CheckpointStreamFooter::decode_record(&bytes[footer_start..])
            .unwrap();
    let mut offset = footer_start - footer.certificate_record_bytes() as usize;
    let (mut tier, mut no_release, mut batch, mut accumulator) = (0, 0, 0, 0);
    for _ in 0..footer.certificate_record_count() {
        let length = checkpoint_certificate_frame_bytes(
            &bytes[offset..offset + CHECKPOINT_CERTIFICATE_PREFIX_BYTES],
        )
        .unwrap();
        let (kind, payload) =
            decode_checkpoint_certificate(&bytes[offset..offset + length]).unwrap();
        match kind {
            worth_store_physical_format::CheckpointCertificateKind::TierEpoch => {
                let certificate =
                    worth_store_physical_format::TierEpochCheckpointCertificateV1::decode(payload)
                        .unwrap();
                assert!(certificate.intent().tier_epoch_start() > 0);
                tier += 1;
            }
            worth_store_physical_format::CheckpointCertificateKind::ReleasedDrop => {
                match worth_store_physical_format::ReleaseCheckpointCertificateV1::decode(payload)
                    .unwrap()
                {
                    worth_store_physical_format::ReleaseCheckpointCertificateV1::NoRelease(_) => {
                        no_release += 1
                    }
                    worth_store_physical_format::ReleaseCheckpointCertificateV1::Batch(_) => {
                        batch += 1
                    }
                    worth_store_physical_format::ReleaseCheckpointCertificateV1::AccumulatorV2(
                        _,
                    ) => accumulator += 1,
                    worth_store_physical_format::ReleaseCheckpointCertificateV1::Accumulator(_) => {
                        panic!("released tier checkpoint cannot use headless V1 accumulator")
                    }
                }
            }
        }
        offset += length;
    }
    assert_eq!(offset, footer_start);
    let observed = (tier, no_release, batch, accumulator);
    assert_eq!(
        observed,
        if folded_batch {
            (1, 0, 1, 1)
        } else {
            (1, 1, 0, 0)
        }
    );
}
