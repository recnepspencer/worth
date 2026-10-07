use std::ffi::OsString;

use worth_store_physical_format::RecordArtifactFile;

use crate::filesystem_media::{ArtifactTreeDirectory, ArtifactTreeFile};

use super::super::ceiling::{CeilingArtifact, StreamArtifact};
use super::RecoveryDiscoveryFailure;

mod backed;
pub(super) use backed::{backed_ceiling_artifact, backed_record_artifact};

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
    let (mut directory, components) = record_directory(artifact);
    for component in components {
        directory = directory
            .child(component)
            .map_err(|_| RecoveryDiscoveryFailure::invalid(context.clone()))?;
    }
    directory
        .file(artifact.canonical_file_name().as_str())
        .map_err(|_| RecoveryDiscoveryFailure::invalid(context))
}

/// The tree file a ceiling names.
pub(super) fn ceiling_artifact(
    address: CeilingArtifact,
) -> Result<ArtifactTreeFile, RecoveryDiscoveryFailure> {
    match address {
        CeilingArtifact::Record(file) => record_artifact(file),
        CeilingArtifact::Stream(StreamArtifact::CurrentCheckpoint) => {
            ArtifactTreeDirectory::families()
                .file(CHECKPOINT_FILE)
                .map_err(|_| {
                    RecoveryDiscoveryFailure::invalid(RecoveryDiscoveryArtifact::CurrentCheckpoint)
                })
        }
    }
}

/// The current checkpoint stream's file in the families directory.
const CHECKPOINT_FILE: &str = "checkpoint.current";

/// Directory classification has one owner; both read-storage modes consume it.
fn record_directory(
    artifact: RecordArtifactFile,
) -> (ArtifactTreeDirectory, &'static [&'static str]) {
    match artifact {
        RecordArtifactFile::BootstrapCatalog
        | RecordArtifactFile::CurrentRootSelector
        | RecordArtifactFile::PreviousRootSelector => {
            (ArtifactTreeDirectory::families(), &["records"])
        }
        RecordArtifactFile::RootSelectorCandidate { .. }
        | RecordArtifactFile::CatalogCandidate { .. } => {
            (ArtifactTreeDirectory::staging(), &["records"])
        }
        RecordArtifactFile::RootManifest { .. }
        | RecordArtifactFile::RootRoutingBlock { .. }
        | RecordArtifactFile::ReleaseCustodyHeadBlock { .. } => {
            (ArtifactTreeDirectory::families(), &["records", "roots"])
        }
        RecordArtifactFile::Segment { .. } => {
            (ArtifactTreeDirectory::families(), &["records", "segments"])
        }
        RecordArtifactFile::SegmentManifest { .. }
        | RecordArtifactFile::SegmentMembershipBlock { .. } => (
            ArtifactTreeDirectory::families(),
            &["records", "segment-manifests"],
        ),
        RecordArtifactFile::ExtentArena { .. } => {
            (ArtifactTreeDirectory::families(), &["records", "arenas"])
        }
        RecordArtifactFile::FreeSpaceManifest { .. }
        | RecordArtifactFile::FreeSpaceMembershipBlock { .. } => (
            ArtifactTreeDirectory::families(),
            &["records", "free-space"],
        ),
    }
}
