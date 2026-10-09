use super::*;
use worth_store_physical_format::{
    checkpoint_certificate_frame_bytes, decode_checkpoint_certificate, CheckpointCertificateKind,
    CheckpointStreamFooter, ReleaseCheckpointCertificateV1, TierEpochCheckpointCertificateV1,
    CHECKPOINT_CERTIFICATE_PREFIX_BYTES, CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES,
};

pub(super) fn selected_checkpoint(root: &Path) -> Vec<u8> {
    fs::read(root.join("families/checkpoint.current")).expect("selected checkpoint")
}

pub(super) fn assert_tier_and_no_release(bytes: &[u8]) {
    assert_tier_release_roster(bytes, false);
}

pub(super) fn assert_tier_and_batch(bytes: &[u8]) {
    assert_tier_release_roster(bytes, true);
}

fn assert_tier_release_roster(bytes: &[u8], folded_batch: bool) {
    let footer_start = bytes.len() - CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES;
    let footer = CheckpointStreamFooter::decode_record(&bytes[footer_start..]).unwrap();
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
            CheckpointCertificateKind::TierEpoch => {
                let certificate = TierEpochCheckpointCertificateV1::decode(payload).unwrap();
                assert!(certificate.intent().tier_epoch_start() > 0);
                tier += 1;
            }
            CheckpointCertificateKind::ReleasedDrop => {
                match ReleaseCheckpointCertificateV1::decode(payload).unwrap() {
                    ReleaseCheckpointCertificateV1::NoRelease(_) => no_release += 1,
                    ReleaseCheckpointCertificateV1::Batch(_) => batch += 1,
                    ReleaseCheckpointCertificateV1::AccumulatorV2(_) => accumulator += 1,
                    ReleaseCheckpointCertificateV1::Accumulator(_) => {
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
