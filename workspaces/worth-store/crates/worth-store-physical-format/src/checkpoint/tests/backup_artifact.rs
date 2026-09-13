use super::super::{CheckpointBackupArtifact, CheckpointBackupArtifactInput};
use crate::{
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId, PhysicalReferenceAuthority,
    PhysicalRootReference, PhysicalSegmentId,
};

#[test]
fn constructor_rejects_a_stale_final_page_frontier() {
    let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
    let pages = vec![
        (
            generations
                .page_cell(
                    PhysicalSegmentId::from_raw(1).expect("segment"),
                    PhysicalPageId::from_raw(1).expect("page"),
                )
                .with_page_generation(PhysicalGeneration::from_raw(1).expect("generation")),
            10,
        ),
        (
            generations
                .page_cell(
                    PhysicalSegmentId::from_raw(1).expect("segment"),
                    PhysicalPageId::from_raw(2).expect("page"),
                )
                .with_page_generation(PhysicalGeneration::from_raw(1).expect("generation")),
            9,
        ),
    ];
    let root_reference = PhysicalRootReference::from_raw(1).expect("root reference");
    let root = PhysicalReferenceAuthority::for_canonical_physical_format()
        .admit_root_publication(
            generations
                .root_publication_cell(root_reference)
                .with_root_publication_generation(
                    PhysicalGeneration::from_raw(1).expect("generation"),
                ),
        )
        .reference();

    assert!(
        CheckpointBackupArtifact::from_input(CheckpointBackupArtifactInput {
            checkpoint_identity: "checkpoint".to_owned(),
            manifest_generation: 3,
            durable_checkpoint_lsn: 10,
            root,
            covered_lsn: (1, 11),
            redo_lsn: 10,
            pages,
        })
        .is_none()
    );
}
