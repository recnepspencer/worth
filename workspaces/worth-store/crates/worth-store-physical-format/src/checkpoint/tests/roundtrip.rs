use sha2::Digest;

use crate::{RecordArtifactFile, RecordFrameCoordinate};

use super::super::{
    inspect_checkpoint_stream, CheckpointBindingCompactionHeader, CheckpointDirtyFrameBasis,
    CheckpointStreamDecoder, CheckpointStreamEncoder, PersistedCompactionProductRole,
};
use super::{secured_source, source};

#[test]
fn security_binding_roundtrips_as_checkpoint_owned_persisted_truth() {
    let source = secured_source(9);
    let (encoder, header) = CheckpointStreamEncoder::begin(source);
    let cutover =
        CheckpointBindingCompactionHeader::new(4, source.wal().covered_end_lsn_exclusive())
            .unwrap();
    let (compaction, cutover_record) = encoder.begin_binding_compaction(cutover);
    let (_, footer) = compaction.finish();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(&cutover_record);
    bytes.extend_from_slice(&footer);
    let verified = inspect_checkpoint_stream(&bytes, 0, 0).unwrap();
    let security = verified.source().security_binding().unwrap();
    assert_eq!(security.policy_identity(), [9; 32]);
    assert_eq!(security.idempotency_retention_generations(), 8);
    assert_ne!(security.digest(), [0; 32]);
}

#[test]
fn maintenance_checkpoint_is_a_distinct_envelope() {
    use crate::PhysicalCheckpointSource;
    let source = source(4).with_maintenance_protocol();
    let (_, header) = CheckpointStreamEncoder::begin(source);
    assert_eq!(header[8], 2);
    assert!(PhysicalCheckpointSource::decode_c9_legacy_stream_header_record(&header).is_err());
    assert!(
        PhysicalCheckpointSource::decode_stream_header_record(&header)
            .unwrap()
            .requires_maintenance_protocol()
    );
}

#[test]
fn independently_framed_stream_roundtrips_without_whole_artifact_memory() {
    let source = source(3);
    let bases = [
        CheckpointDirtyFrameBasis::new(
            RecordFrameCoordinate::new(
                RecordArtifactFile::Segment {
                    segment: 2,
                    generation: 9,
                },
                128,
                64,
            )
            .unwrap(),
            17,
        ),
        CheckpointDirtyFrameBasis::new(
            RecordFrameCoordinate::new(
                RecordArtifactFile::RootRoutingBlock {
                    generation: 5,
                    block: 4,
                },
                0,
                96,
            )
            .unwrap(),
            19,
        ),
    ];
    let (mut encoder, header) = CheckpointStreamEncoder::begin(source);
    let records = bases.map(|basis| encoder.encode_dirty_basis(basis));
    let compaction_header =
        CheckpointBindingCompactionHeader::new(1, source.wal().covered_end_lsn_exclusive())
            .unwrap();
    let (mut compaction, compaction_record) = encoder.begin_binding_compaction(compaction_header);
    let binding_payloads = [b"binding-one".as_slice(), b"binding-two".as_slice()];
    let binding_records = binding_payloads.map(|payload| {
        compaction
            .encode_binding_record(payload)
            .expect("bounded binding record")
    });
    let (encoded_footer, footer) = compaction.finish();

    let mut decoder = CheckpointStreamDecoder::begin(&header).unwrap();
    assert_eq!(decoder.source(), source);
    for (record, expected) in records.iter().zip(bases) {
        assert_eq!(decoder.decode_dirty_basis(record).unwrap(), expected);
    }
    let mut compaction = decoder
        .begin_binding_compaction(&compaction_record)
        .unwrap();
    assert_eq!(compaction.header(), compaction_header);
    for (record, expected) in binding_records.iter().zip(binding_payloads) {
        assert_eq!(compaction.decode_binding_record(record).unwrap(), expected);
    }
    let decoded_footer = compaction.finish(&footer).unwrap();
    assert_eq!(decoded_footer, encoded_footer);
    assert_eq!(decoded_footer.dirty_record_count(), 2);
    assert_eq!(decoded_footer.binding_record_count(), 2);
    assert_eq!(decoded_footer.identity(), source.identity());
}

#[test]
fn whole_stream_inspection_binds_the_raw_compaction_cutover_to_its_checkpoint() {
    let source = source(7);
    let (encoder, header) = CheckpointStreamEncoder::begin(source);
    let compaction_header =
        CheckpointBindingCompactionHeader::new(3, source.wal().covered_end_lsn_exclusive())
            .unwrap();
    let (mut compaction, compaction_record) = encoder.begin_binding_compaction(compaction_header);
    let binding = compaction.encode_binding_record(b"binding").unwrap();
    let (_, footer) = compaction.finish();
    let mut artifact = Vec::new();
    artifact.extend_from_slice(&header);
    artifact.extend_from_slice(&compaction_record);
    artifact.extend_from_slice(&binding);
    artifact.extend_from_slice(&footer);

    let verified = inspect_checkpoint_stream(&artifact, 0, 1).unwrap();
    let cutover = verified.compaction_cutover();
    assert_eq!(verified.source(), source);
    assert_eq!(cutover.checkpoint(), source.identity());
    assert_eq!(cutover.root(), source.root());
    assert_eq!(cutover.checkpoint_wal(), source.wal());
    assert_eq!(
        cutover.product_role(),
        PersistedCompactionProductRole::OperationBindingIndex
    );
    assert_eq!(cutover.product_generation(), 3);
    assert_eq!(
        cutover.wal_cutoff_lsn_exclusive(),
        source.wal().covered_end_lsn_exclusive()
    );
    assert_eq!(verified.footer().binding_record_count(), 1);
    assert_eq!(verified.footer().identity(), source.identity());
    assert_eq!(verified.encoded_bytes(), artifact.len() as u64);
    let digest: [u8; 32] = sha2::Sha256::digest(&artifact).into();
    assert_eq!(verified.encoded_digest(), digest);
    assert_eq!(
        verified.binding_records(),
        [Box::<[u8]>::from(&b"binding"[..])]
    );
    assert!(inspect_checkpoint_stream(&artifact, 0, 0).is_err());
}
