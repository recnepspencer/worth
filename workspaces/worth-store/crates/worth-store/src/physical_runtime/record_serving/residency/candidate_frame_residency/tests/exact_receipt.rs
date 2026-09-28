use worth_proof::TransitionOutcome;
use worth_store_physical_backend::{
    ArtifactNewWriteOutcome, ArtifactNewWriteRange, ArtifactTreeDirectory, FilesystemAccessPosture,
};
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use crate::physical_runtime::{
    FilesystemMediaAdmission, MediaOwnedPhysicalRuntime, MediaShutdownOutcome,
    PhysicalRuntimeAdmission, PhysicalStore,
};

#[test]
fn foreign_real_receipt_cannot_settle_candidate_residency() {
    let parent = tempfile::tempdir().unwrap();
    let owned = admit_store_media(&parent.path().join("store"));
    let media = owned.record_serving_media();
    let (artifact, physical) = root_manifest_artifact(media);
    let coordinate = RecordFrameCoordinate::new(artifact, 0, 8).unwrap();
    let receipt = match media.artifact_tree().write_new_exact(
        &physical,
        ArtifactNewWriteRange::new(8).unwrap(),
        &[0xA5; 8],
    ) {
        ArtifactNewWriteOutcome::Completed(completed) => completed,
        outcome => panic!("real receipt write failed: {outcome:?}"),
    };
    assert!(
        !super::super::write_evidence::completed_new_artifact_matches(
            &receipt,
            coordinate,
            media.store_identity(),
            coordinate,
            &[0x5A; 8],
        ),
        "C5_PREDICATE:candidate-clean-without-exact-receipt"
    );
    let displaced = RecordFrameCoordinate::new(artifact, 4096, 8).unwrap();
    assert!(
        !super::super::write_evidence::completed_new_artifact_matches(
            &receipt,
            displaced,
            media.store_identity(),
            displaced,
            &[0xA5; 8],
        ),
        "a receipt at file zero cannot settle a candidate at another arena offset"
    );
    assert!(matches!(owned.close(), MediaShutdownOutcome::Released(_)));
}

fn admit_store_media(root: &std::path::Path) -> MediaOwnedPhysicalRuntime {
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    match runtime.try_admit_filesystem_media(admission).into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("Store-owned backend admission failed"),
    }
}

fn root_manifest_artifact(
    media: &worth_store_physical_backend::QualifiedFilesystemMedia,
) -> (
    RecordArtifactFile,
    worth_store_physical_backend::ArtifactTreeFile,
) {
    let tree = media.artifact_tree();
    let records = ArtifactTreeDirectory::families().child("records").unwrap();
    let roots = records.child("roots").unwrap();
    if !tree.directory_exists(&records).unwrap() {
        tree.create_directory(&records).unwrap();
    }
    if !tree.directory_exists(&roots).unwrap() {
        tree.create_directory(&roots).unwrap();
    }
    let logical = RecordArtifactFile::RootManifest { generation: 1 };
    let physical = roots.file(&logical.file_name()).unwrap();
    (logical, physical)
}
