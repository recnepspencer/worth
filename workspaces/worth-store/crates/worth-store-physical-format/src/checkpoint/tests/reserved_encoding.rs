use super::{secured_source, source};
use crate::{
    decode_checkpoint_certificate, encode_checkpoint_certificate,
    encode_checkpoint_certificate_in_reserved, CheckpointBindingCompactionEncoder,
    CheckpointBindingCompactionHeader, CheckpointCertificateKind, CheckpointDirtyFrameBasis,
    CheckpointStreamDecodeDenial, CheckpointStreamEncoder, CheckpointStreamEncodingDenial,
    RecordArtifactFile, RecordFrameCoordinate, CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES,
    CHECKPOINT_CERTIFICATE_RECORD_OVERHEAD_BYTES, CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES,
    CHECKPOINT_DIRTY_FRAME_RECORD_BYTES, CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
    MAX_CHECKPOINT_CERTIFICATE_BYTES, MAX_CHECKPOINT_CERTIFICATE_RECORDS,
};

fn dirty() -> CheckpointDirtyFrameBasis {
    CheckpointDirtyFrameBasis::new(
        RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 64, 16).unwrap(),
        6,
    )
}

fn cutover() -> CheckpointBindingCompactionHeader {
    CheckpointBindingCompactionHeader::new(9, 29).unwrap()
}

fn compaction() -> CheckpointBindingCompactionEncoder {
    CheckpointStreamEncoder::begin_certified(source(7))
        .0
        .begin_binding_compaction(cutover())
        .0
}

fn small_buffer() -> Vec<u8> {
    vec![0xa5]
}

fn assert_shortage(denial: CheckpointStreamEncodingDenial, required: usize, buffer: &Vec<u8>) {
    assert_eq!(
        denial,
        CheckpointStreamEncodingDenial::InsufficientReservedCapacity {
            required,
            capacity: buffer.capacity(),
        }
    );
    assert_eq!(buffer.as_slice(), [0xa5]);
}

#[test]
fn certified_commands_reuse_one_backing_and_roundtrip_the_same_canonical_stream() {
    let source = secured_source(7);
    let (mut ordinary, header) = CheckpointStreamEncoder::begin_certified(source);
    let dirty_record = ordinary.encode_dirty_basis(dirty());
    let (mut ordinary, compact) = ordinary.begin_binding_compaction(cutover());
    let binding = ordinary.encode_binding_record(b"binding").unwrap();
    let tier = ordinary
        .encode_certificate_record(CheckpointCertificateKind::TierEpoch, b"tier")
        .unwrap();
    let release = ordinary
        .encode_certificate_record(CheckpointCertificateKind::ReleasedDrop, b"release")
        .unwrap();
    let (expected_footer, footer) = ordinary.finish();

    let mut buffer = Vec::with_capacity(4116);
    let capacity = buffer.capacity();
    let pointer = buffer.as_ptr();
    let mut encoder =
        CheckpointStreamEncoder::begin_certified_in_reserved(source, &mut buffer).unwrap();
    assert_eq!(buffer, header);
    let mut artifact = buffer.clone();
    encoder
        .encode_dirty_basis_in_reserved(dirty(), &mut buffer)
        .unwrap();
    assert_eq!(buffer, dirty_record);
    artifact.extend_from_slice(&buffer);
    let mut encoder = encoder
        .begin_binding_compaction_in_reserved(cutover(), &mut buffer)
        .unwrap();
    assert_eq!(buffer, compact);
    artifact.extend_from_slice(&buffer);
    encoder
        .encode_binding_record_in_reserved(b"binding", &mut buffer)
        .unwrap();
    assert_eq!(buffer, binding);
    artifact.extend_from_slice(&buffer);
    for (kind, payload, expected) in [
        (
            CheckpointCertificateKind::TierEpoch,
            b"tier".as_slice(),
            tier,
        ),
        (
            CheckpointCertificateKind::ReleasedDrop,
            b"release".as_slice(),
            release,
        ),
    ] {
        encode_checkpoint_certificate_in_reserved(kind, payload, &mut buffer).unwrap();
        assert_eq!(buffer, expected);
        assert_eq!(decode_checkpoint_certificate(&buffer), Ok((kind, payload)));
        encoder.include_certificate_record(&buffer).unwrap();
        artifact.extend_from_slice(&buffer);
    }
    assert_eq!(encoder.finish_in_reserved(&mut buffer), Ok(expected_footer));
    assert_eq!(buffer, footer);
    artifact.extend_from_slice(&buffer);
    assert_eq!(buffer.capacity(), capacity);
    assert_eq!(buffer.as_ptr(), pointer);
    let inspected = super::super::inspect_checkpoint_stream(&artifact, 1, 1).unwrap();
    assert_eq!(inspected.footer(), expected_footer);
}

#[test]
fn insufficient_command_backing_denies_without_growth_or_aggregate_inclusion() {
    let mut buffer = small_buffer();
    let capacity = buffer.capacity();
    let pointer = buffer.as_ptr();
    assert_shortage(
        CheckpointStreamEncoder::begin_certified_in_reserved(source(7), &mut buffer).unwrap_err(),
        CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
        &buffer,
    );
    let mut encoder = CheckpointStreamEncoder::begin_certified(source(7)).0;
    assert_shortage(
        encoder
            .encode_dirty_basis_in_reserved(dirty(), &mut buffer)
            .unwrap_err(),
        CHECKPOINT_DIRTY_FRAME_RECORD_BYTES,
        &buffer,
    );
    assert_shortage(
        encoder
            .begin_binding_compaction_in_reserved(cutover(), &mut buffer)
            .unwrap_err(),
        CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES,
        &buffer,
    );
    let mut encoder = compaction();
    assert_shortage(
        encoder
            .encode_binding_record_in_reserved(b"binding", &mut buffer)
            .unwrap_err(),
        20 + b"binding".len(),
        &buffer,
    );
    assert_shortage(
        encode_checkpoint_certificate_in_reserved(
            CheckpointCertificateKind::ReleasedDrop,
            b"release",
            &mut buffer,
        )
        .unwrap_err(),
        CHECKPOINT_CERTIFICATE_RECORD_OVERHEAD_BYTES + b"release".len(),
        &buffer,
    );
    let mut footer_buffer = Vec::with_capacity(CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES);
    let footer = encoder.finish_in_reserved(&mut footer_buffer).unwrap();
    assert_eq!(footer.binding_record_count(), 0);
    assert_eq!(footer.certificate_record_count(), 0);
    assert_shortage(
        compaction().finish_in_reserved(&mut buffer).unwrap_err(),
        CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES,
        &buffer,
    );
    assert_eq!(buffer.capacity(), capacity);
    assert_eq!(buffer.as_ptr(), pointer);
}

#[test]
fn reserved_encoding_preserves_format_denial_without_polluting_decode_outcomes() {
    let mut buffer = small_buffer();
    let mut encoder = compaction();
    assert_eq!(
        encoder.encode_binding_record_in_reserved(&[], &mut buffer),
        Err(CheckpointStreamEncodingDenial::Format(
            CheckpointStreamDecodeDenial::EmptyBindingRecord,
        )),
    );
    assert_eq!(
        encoder.encode_binding_record(&[]),
        Err(CheckpointStreamDecodeDenial::EmptyBindingRecord),
    );
    assert_eq!(
        encode_checkpoint_certificate_in_reserved(
            CheckpointCertificateKind::ReleasedDrop,
            &[],
            &mut buffer,
        ),
        Err(CheckpointStreamEncodingDenial::Format(
            CheckpointStreamDecodeDenial::EmptyBindingRecord,
        )),
    );
    assert_eq!(buffer.as_slice(), [0xa5]);
}

#[test]
fn framed_certificate_inclusion_checks_integrity_order_and_section_bounds_before_commit() {
    let tier =
        encode_checkpoint_certificate(CheckpointCertificateKind::TierEpoch, b"tier").unwrap();
    let release =
        encode_checkpoint_certificate(CheckpointCertificateKind::ReleasedDrop, b"r").unwrap();
    let mut encoder = compaction();
    let mut corrupt = tier.clone();
    *corrupt.last_mut().unwrap() ^= 1;
    assert_eq!(
        encoder.include_certificate_record(&corrupt),
        Err(CheckpointStreamDecodeDenial::IntegrityMismatch)
    );
    assert_eq!(
        encoder.include_certificate_record(&tier[..tier.len() - 1]),
        Err(CheckpointStreamDecodeDenial::LengthMismatch)
    );
    let mut wrong_kind = tier.clone();
    wrong_kind[9] = 4;
    assert_eq!(
        encoder.include_certificate_record(&wrong_kind),
        Err(CheckpointStreamDecodeDenial::InvalidArtifactKind(4))
    );
    encoder.include_certificate_record(&tier).unwrap();
    assert_eq!(
        encoder.include_certificate_record(&tier),
        Err(CheckpointStreamDecodeDenial::RecordCountMismatch)
    );
    for _ in 1..MAX_CHECKPOINT_CERTIFICATE_RECORDS {
        encoder.include_certificate_record(&release).unwrap();
    }
    assert_eq!(
        encoder.include_certificate_record(&release),
        Err(CheckpointStreamDecodeDenial::RecordByteCountMismatch)
    );
    assert_eq!(
        encoder.finish().0.certificate_record_count(),
        MAX_CHECKPOINT_CERTIFICATE_RECORDS
    );

    let mut encoder = compaction();
    encoder.include_certificate_record(&release).unwrap();
    assert_eq!(
        encoder.include_certificate_record(&tier),
        Err(CheckpointStreamDecodeDenial::BindingCompactionMismatch)
    );
    let maximum = encode_checkpoint_certificate(
        CheckpointCertificateKind::ReleasedDrop,
        &vec![
            1;
            MAX_CHECKPOINT_CERTIFICATE_BYTES as usize
                - CHECKPOINT_CERTIFICATE_RECORD_OVERHEAD_BYTES
        ],
    )
    .unwrap();
    let mut encoder = compaction();
    encoder.include_certificate_record(&maximum).unwrap();
    assert_eq!(
        encoder.include_certificate_record(&release),
        Err(CheckpointStreamDecodeDenial::RecordByteCountMismatch)
    );
    assert_eq!(
        encoder.finish().0.certificate_record_bytes(),
        MAX_CHECKPOINT_CERTIFICATE_BYTES
    );
}
