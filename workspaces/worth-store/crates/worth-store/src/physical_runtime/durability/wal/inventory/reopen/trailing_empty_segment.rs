use worth_store_physical_backend::{ArtifactTreeDirectory, ArtifactTreeFile, ArtifactTreeMedia};
use worth_store_wal::{WalSegmentArtifactIdentity, WalTopologyDenialKind, WAL_ORIGIN};

use super::{artifact, PhysicalWalOpenFailure};

pub(super) struct TrailingEmptySegment {
    artifact: ArtifactTreeFile,
}

pub(super) fn separate(
    tree: &ArtifactTreeMedia<'_>,
    directory: &ArtifactTreeDirectory,
    segments: &mut Vec<WalSegmentArtifactIdentity>,
) -> Result<Option<TrailingEmptySegment>, PhysicalWalOpenFailure> {
    let trailing = *segments
        .last()
        .expect("reopen calls trailing cleanup only for a nonempty inventory");
    let trailing_artifact = artifact(directory, trailing);
    let trailing_bytes = tree
        .file_length(&trailing_artifact)
        .map_err(PhysicalWalOpenFailure::Media)?;
    if trailing_bytes != 0 {
        return Ok(None);
    }
    if segments.len() == 1 {
        // The origin segment was created but no frame reached it: the WAL
        // holds nothing, exactly as if the segment were absent. Any other
        // sole empty segment has lost the segments before it.
        if trailing != WAL_ORIGIN.artifact() {
            return Err(PhysicalWalOpenFailure::EmptySegment);
        }
        segments.pop();
        return Ok(Some(TrailingEmptySegment {
            artifact: trailing_artifact,
        }));
    }

    let previous = segments[segments.len() - 2];
    if trailing.generation() != previous.generation() {
        return Err(PhysicalWalOpenFailure::Topology(
            WalTopologyDenialKind::WrongGeneration,
        ));
    }
    let expected_segment = previous.segment().get().checked_add(1);
    if expected_segment != Some(trailing.segment().get()) {
        return Err(PhysicalWalOpenFailure::Topology(
            WalTopologyDenialKind::NonContiguousSegment,
        ));
    }

    segments.pop();
    Ok(Some(TrailingEmptySegment {
        artifact: trailing_artifact,
    }))
}

impl TrailingEmptySegment {
    pub(super) fn remove_durably(
        self,
        tree: &ArtifactTreeMedia<'_>,
    ) -> Result<(), PhysicalWalOpenFailure> {
        tree.remove_file_durably(&self.artifact)
            .map_err(PhysicalWalOpenFailure::Media)
    }
}
