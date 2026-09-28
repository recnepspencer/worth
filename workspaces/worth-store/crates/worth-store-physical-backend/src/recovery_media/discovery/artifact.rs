use std::ffi::OsString;

use worth_store_physical_format::RecordArtifactFile;

use crate::filesystem_media::{ArtifactTreeDirectory, ArtifactTreeFile};

use super::RecoveryDiscoveryFailure;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryDiscoveryArtifact {
    Record(RecordArtifactFile),
    CurrentCheckpoint,
    WalDirectory,
    WalArtifact(OsString),
}

pub(crate) fn record_artifact(
    artifact: RecordArtifactFile,
) -> Result<ArtifactTreeFile, RecoveryDiscoveryFailure> {
    let context = RecoveryDiscoveryArtifact::Record(artifact);
    let records = ArtifactTreeDirectory::families()
        .child("records")
        .map_err(|_| RecoveryDiscoveryFailure::invalid(context.clone()))?;
    let directory =
        match artifact {
            RecordArtifactFile::BootstrapCatalog
            | RecordArtifactFile::CurrentRootSelector
            | RecordArtifactFile::PreviousRootSelector => records,
            RecordArtifactFile::RootSelectorCandidate { .. }
            | RecordArtifactFile::CatalogCandidate { .. } => ArtifactTreeDirectory::staging()
                .child("records")
                .map_err(|_| RecoveryDiscoveryFailure::invalid(context.clone()))?,
            RecordArtifactFile::RootManifest { .. }
            | RecordArtifactFile::RootRoutingBlock { .. } => records
                .child("roots")
                .map_err(|_| RecoveryDiscoveryFailure::invalid(context.clone()))?,
            RecordArtifactFile::Segment { .. } => records
                .child("segments")
                .map_err(|_| RecoveryDiscoveryFailure::invalid(context.clone()))?,
            RecordArtifactFile::SegmentManifest { .. }
            | RecordArtifactFile::SegmentMembershipBlock { .. } => records
                .child("segment-manifests")
                .map_err(|_| RecoveryDiscoveryFailure::invalid(context.clone()))?,
            RecordArtifactFile::ExtentArena { .. } => records
                .child("arenas")
                .map_err(|_| RecoveryDiscoveryFailure::invalid(context.clone()))?,
            RecordArtifactFile::FreeSpaceManifest { .. }
            | RecordArtifactFile::FreeSpaceMembershipBlock { .. } => records
                .child("free-space")
                .map_err(|_| RecoveryDiscoveryFailure::invalid(context.clone()))?,
        };
    directory
        .file(&artifact.file_name())
        .map_err(|_| RecoveryDiscoveryFailure::invalid(context))
}
