use std::num::NonZeroU64;

use worth_store_offline_verifier::{
    observe_recovery_artifacts, RecoveryObserverLimits, RecoveryObserverReport,
};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    CheckpointBindingCompactionHeader, CheckpointDirtyFrameBasis, CheckpointRootBasis,
    CheckpointStreamEncoder, CheckpointWalSourceRange, PhysicalCheckpointIdentity,
    PhysicalCheckpointSource, RecordArtifactFile, RecordFrameCoordinate,
};

#[test]
fn real_observer_entry_admits_a_complete_checkpoint_stream() {
    let fixture = checkpoint_fixture(0);
    let root = write_checkpoint(&fixture.bytes);

    let report = observe(root.path(), fixture.bytes.len());

    assert_eq!(report.artifact_count(), 1);
    assert_eq!(report.bytes_read(), fixture.bytes.len() as u64);
    assert_eq!(report.checkpoint_count(), 1);
    assert_eq!(report.checkpoint_page_count(), 1);
    assert_eq!(report.checkpoint_covered_lsn_start(), Some(11));
    assert_eq!(report.checkpoint_covered_lsn_end(), Some(29));
    assert_eq!(report.checkpoint_redo_lsn(), Some(11));
    assert_eq!(report.durable_checkpoint_lsn(), Some(29));
    assert_eq!(report.residue_artifact_count(), 0);
    assert_eq!(report.residue_bytes(), 0);
}

#[test]
fn real_observer_entry_rejects_checksum_valid_dirty_mutation_with_stale_aggregate() {
    let mut fixture = checkpoint_fixture(0);
    let substitute = checkpoint_fixture(1);
    fixture.bytes[fixture.dirty_offset..fixture.dirty_offset + fixture.dirty_bytes]
        .copy_from_slice(
            &substitute.bytes
                [substitute.dirty_offset..substitute.dirty_offset + substitute.dirty_bytes],
        );
    assert_residue(fixture);
}

#[test]
fn real_observer_entry_rejects_checksum_valid_binding_mutation_with_stale_aggregate() {
    let mut fixture = checkpoint_fixture(0);
    let substitute = checkpoint_fixture(1);
    fixture.bytes[fixture.binding_offset..fixture.binding_offset + fixture.binding_bytes]
        .copy_from_slice(
            &substitute.bytes
                [substitute.binding_offset..substitute.binding_offset + substitute.binding_bytes],
        );
    assert_residue(fixture);
}

fn assert_residue(fixture: CheckpointFixture) {
    let root = write_checkpoint(&fixture.bytes);
    let report = observe(root.path(), fixture.bytes.len());
    assert_eq!(report.artifact_count(), 1);
    assert_eq!(report.bytes_read(), fixture.bytes.len() as u64);
    assert_eq!(report.checkpoint_count(), 0);
    assert_eq!(report.checkpoint_page_count(), 0);
    assert_eq!(report.residue_artifact_count(), 1);
    assert_eq!(report.residue_bytes(), fixture.bytes.len() as u64);
}

fn observe(root: &std::path::Path, artifact_bytes: usize) -> RecoveryObserverReport {
    observe_recovery_artifacts(
        root,
        RecoveryObserverLimits::new(2, 2, 1, artifact_bytes as u64).unwrap(),
    )
    .unwrap()
}

struct CheckpointFixture {
    bytes: Vec<u8>,
    dirty_offset: usize,
    dirty_bytes: usize,
    binding_offset: usize,
    binding_bytes: usize,
}

fn checkpoint_fixture(variant: u8) -> CheckpointFixture {
    let proposed = ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap();
    let store = StoreNamespaceIdentityRecord::new(StoreNamespaceVersion::CURRENT, proposed)
        .published_identity();
    let identity = PhysicalCheckpointIdentity::new(store, NonZeroU64::new(3).unwrap());
    let source = PhysicalCheckpointSource::concurrent(
        identity,
        CheckpointWalSourceRange::new(11, 29).unwrap(),
        CheckpointRootBasis::new(5, 71),
        43,
    );
    let (mut encoder, header) = CheckpointStreamEncoder::begin(source);
    let coordinate = RecordFrameCoordinate::new(
        RecordArtifactFile::Segment {
            segment: 2,
            generation: 9,
        },
        128,
        64,
    )
    .unwrap();
    let dirty = encoder.encode_dirty_basis(CheckpointDirtyFrameBasis::new(
        coordinate,
        17 + u64::from(variant),
    ));
    let (mut compaction, compaction_header) =
        encoder.begin_binding_compaction(CheckpointBindingCompactionHeader::new(4, 29).unwrap());
    let mut binding_payload = *b"observer-binding-a";
    binding_payload[17] = b'a' + variant;
    let binding = compaction.encode_binding_record(&binding_payload).unwrap();
    let (_, footer) = compaction.finish();
    let dirty_offset = header.len();
    let binding_offset = dirty_offset + dirty.len() + compaction_header.len();
    let mut bytes = header;
    bytes.extend_from_slice(&dirty);
    bytes.extend_from_slice(&compaction_header);
    bytes.extend_from_slice(&binding);
    bytes.extend_from_slice(&footer);
    CheckpointFixture {
        bytes,
        dirty_offset,
        dirty_bytes: dirty.len(),
        binding_offset,
        binding_bytes: binding.len(),
    }
}

fn write_checkpoint(bytes: &[u8]) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let families = root.path().join("families");
    std::fs::create_dir(&families).unwrap();
    std::fs::write(families.join("checkpoint.current"), bytes).unwrap();
    root
}
