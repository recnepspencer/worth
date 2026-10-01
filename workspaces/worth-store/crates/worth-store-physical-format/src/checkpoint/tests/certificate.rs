use super::super::{inspect_checkpoint_stream, CheckpointBindingCompactionHeader};
use super::secured_source;
use crate::checkpoint::{
    CheckpointCertificateKind, CheckpointStreamEncoder, MAX_CHECKPOINT_CERTIFICATE_BYTES,
    MAX_CHECKPOINT_CERTIFICATE_RECORDS,
};

fn certified_stream() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let source = secured_source(17).with_maintenance_protocol();
    let (encoder, header) = CheckpointStreamEncoder::begin_certified(source);
    assert_eq!(header[8], 3);
    assert_eq!(
        super::super::PhysicalCheckpointSource::decode_stream_header_record(&header),
        Ok(source)
    );
    let cutover =
        CheckpointBindingCompactionHeader::new(5, source.wal().covered_end_lsn_exclusive())
            .unwrap();
    let (mut encoder, compaction) = encoder.begin_binding_compaction(cutover);
    let tier = encoder
        .encode_certificate_record(CheckpointCertificateKind::TierEpoch, b"exact-tier-custody")
        .unwrap();
    let release = encoder
        .encode_certificate_record(
            CheckpointCertificateKind::ReleasedDrop,
            b"exact-release-custody",
        )
        .unwrap();
    let (footer, end) = encoder.finish();
    assert_eq!(footer.certificate_record_count(), 2);
    assert_eq!(
        footer.certificate_record_bytes(),
        (tier.len() + release.len()) as u64
    );
    assert!(footer.certificate_record_bytes() <= MAX_CHECKPOINT_CERTIFICATE_BYTES);
    let mut prefix = header;
    prefix.extend_from_slice(&compaction);
    prefix.extend_from_slice(&tier);
    prefix.extend_from_slice(&release);
    (prefix, end, b"exact-tier-custody".to_vec())
}

#[test]
fn certified_stream_retains_exact_ordered_purpose_records() {
    let (mut bytes, footer, tier) = certified_stream();
    bytes.extend_from_slice(&footer);
    let verified = inspect_checkpoint_stream(&bytes, 0, 0).unwrap();
    assert_eq!(verified.certificate_records().len(), 2);
    let (kind, payload) =
        crate::checkpoint::decode_checkpoint_certificate(&verified.certificate_records()[0])
            .unwrap();
    assert_eq!(kind, CheckpointCertificateKind::TierEpoch);
    assert_eq!(payload, tier);
    assert_eq!(verified.footer().certificate_record_count(), 2);
}

#[test]
fn certified_footer_corruption_and_truncation_never_fall_back_to_legacy() {
    let (mut prefix, footer, _) = certified_stream();
    prefix.extend_from_slice(&footer);
    assert_eq!(prefix[8], 3);
    let mut corrupt = prefix.clone();
    *corrupt.last_mut().unwrap() ^= 1;
    assert!(inspect_checkpoint_stream(&corrupt, 0, 0).is_err());
    prefix.pop();
    assert!(inspect_checkpoint_stream(&prefix, 0, 0).is_err());
}

#[test]
fn certificate_section_rejects_duplicate_tier_and_hard_count() {
    let source = secured_source(18).with_maintenance_protocol();
    let (encoder, _) = CheckpointStreamEncoder::begin_certified(source);
    let cutover = CheckpointBindingCompactionHeader::new(5, 29).unwrap();
    let (mut encoder, _) = encoder.begin_binding_compaction(cutover);
    encoder
        .encode_certificate_record(CheckpointCertificateKind::TierEpoch, b"one")
        .unwrap();
    assert!(encoder
        .encode_certificate_record(CheckpointCertificateKind::TierEpoch, b"two")
        .is_err());
    for _ in 1..MAX_CHECKPOINT_CERTIFICATE_RECORDS {
        encoder
            .encode_certificate_record(CheckpointCertificateKind::ReleasedDrop, b"r")
            .unwrap();
    }
    assert!(encoder
        .encode_certificate_record(CheckpointCertificateKind::ReleasedDrop, b"extra")
        .is_err());
}
