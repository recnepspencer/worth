#[allow(unused_imports)]
use crate::phase_five_checkpoint_observer::*;

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
