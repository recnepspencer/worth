use super::{candidate_coverage_conflicts, SourceInspectionKind};
use worth_store_physical_format::{BlobReclaimDescriptorV1, BlobRecordV1, PersistedRecordIdentity};

#[test]
fn residue_requires_all_manifest_payloads_absent_while_descriptor_requires_presence() {
    let record = PersistedRecordIdentity::new([1; 16], 1).expect("nonzero identity");
    let residue = SourceInspectionKind::ManifestResidue {
        manifest: record,
        reserved: None,
    };
    let descriptor = SourceInspectionKind::DropDescriptor {
        descriptor: record,
        reservation: None,
    };
    assert!(!candidate_coverage_conflicts(
        residue,
        &[false, false],
        &[None, None],
        0
    ));
    assert!(candidate_coverage_conflicts(
        residue,
        &[false, true],
        &[None, Some(9)],
        0
    ));
    assert!(candidate_coverage_conflicts(
        descriptor,
        &[false, false],
        &[None, None],
        0
    ));
    assert!(!candidate_coverage_conflicts(
        descriptor,
        &[true, true],
        &[Some(9), None],
        9
    ));
}

#[test]
fn residue_rejects_descriptor_for_same_manifest_even_with_different_attempt() {
    let manifest = PersistedRecordIdentity::new([1; 16], 1).unwrap();
    let descriptor_record = PersistedRecordIdentity::new([1; 16], 2).unwrap();
    let descriptor =
        BlobReclaimDescriptorV1::new([2; 16], [3; 16], [4; 32], manifest, [5; 32], 1, 4, 5)
            .unwrap();
    assert!(SourceInspectionKind::ManifestResidue {
        manifest,
        reserved: None
    }
    .conflicting_record(
        descriptor_record,
        &BlobRecordV1::ReclaimDescriptor(descriptor),
        [6; 16],
    ));
}
