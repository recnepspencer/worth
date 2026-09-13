use std::io::Read;
use std::path::Path;

use super::super::owner_artifact_verification::{verify_owner_artifact, OwnerObservation};
use super::super::owner_family_mapping::offline_family;
use super::super::BackupArtifactSemanticDefectKind;
use crate::inspection::OwnerDecodedArtifactBinding;
use crate::truth_composition::RecoveryCandidateObservation;
use worth_store_physical_format::{BackupBundleArtifactManifestRow, RootPublicationCell};

pub(super) struct VerifiedOwnerRow {
    pub(super) observation: OwnerObservation,
    pub(super) recovery_candidate: Option<RecoveryCandidateObservation>,
    pub(super) root_publication: Option<RootPublicationCell>,
    pub(super) owner_binding: OwnerDecodedArtifactBinding,
}

pub(super) fn verify(
    reader: &mut impl Read,
    actual_bytes: u64,
    expected_root: Option<RootPublicationCell>,
    root: &Path,
    row: &BackupBundleArtifactManifestRow,
    max_buffer_bytes: usize,
) -> Result<VerifiedOwnerRow, BackupArtifactSemanticDefectKind> {
    let verified =
        verify_owner_artifact(reader, actual_bytes, expected_root, row, max_buffer_bytes)?;
    let physical_owner = row
        .reclaim_owner()
        .generation_owner()
        .ok_or(BackupArtifactSemanticDefectKind::OwnerReferenceInvalid)?;
    let owner_binding = OwnerDecodedArtifactBinding::with_physical_owner(
        root.join(row.output_name()),
        offline_family(row.family()),
        row.generation(),
        physical_owner,
    )
    .expect("admitted backup row has a nonzero generation");
    let observation = verified.observation();
    let root_publication = verified.root_publication();
    let recovery_candidate = verified.into_recovery_candidate();
    Ok(VerifiedOwnerRow {
        observation,
        recovery_candidate,
        root_publication,
        owner_binding,
    })
}
