use worth_store_physical_backend::ArtifactTreeFailureKind;
use worth_store_physical_format::RecordArtifactFile;

use super::super::residency::serving_artifacts::ServingRecordArtifacts;
use super::bootstrap::{backend_before_effect, BootstrapTransitionFailure};

/// Bytes retained by published inline segment files.
///
/// One publication can seal several segments, and only the last is stored on
/// the root. Every id below the segment frontier is charged for each generation
/// that still has a file. A reserved id that never published is absent and skipped.
pub(super) fn retained_inline_bytes(
    artifacts: &ServingRecordArtifacts,
    next_segment: u64,
    generation_limit: u64,
) -> Result<u64, BootstrapTransitionFailure> {
    let mut retained = 0_u64;
    let mut segment = 1_u64;
    while segment < next_segment {
        let mut generation = 1_u64;
        while generation <= generation_limit {
            match artifacts.file_length(RecordArtifactFile::Segment { segment, generation }) {
                Ok(bytes) => retained = retained.saturating_add(bytes),
                Err(failure) if failure.kind() == ArtifactTreeFailureKind::Absent => {}
                Err(failure) => return Err(backend_before_effect(failure)),
            }
            generation = generation.saturating_add(1);
        }
        segment = segment.saturating_add(1);
    }
    Ok(retained)
}
