use worth_store_physical_format::{DurablePhysicalRootManifest, RecordArtifactFile};

use super::super::residency::serving_artifacts::ServingRecordArtifacts;
use super::bootstrap::{backend_before_effect, BootstrapTransitionFailure, DisplacedSegmentCharge};

/// Rewrite steps in the publication chain whose source segment file remains.
///
/// An append changes the record count, so its predecessor stays out of this
/// charge. A rewrite keeps the count and advances the same segment generation.
/// Retirement removes the source file; the charge ends with the file.
pub(super) fn retained_displaced_segments(
    prior: &[DurablePhysicalRootManifest],
    current: &DurablePhysicalRootManifest,
    artifacts: &ServingRecordArtifacts,
) -> Result<Vec<DisplacedSegmentCharge>, BootstrapTransitionFailure> {
    let mut displaced = Vec::new();
    let mut before = prior.first();
    for root in prior.iter().skip(1).chain(std::iter::once(current)) {
        if let Some(older) = before {
            if let Some(charge) = rewrite_displacement(older, root) {
                let present = artifacts
                    .file_exists(RecordArtifactFile::Segment {
                        segment: charge.segment_id,
                        generation: charge.generation,
                    })
                    .map_err(backend_before_effect)?;
                if present {
                    displaced.push(charge);
                }
            }
        }
        before = Some(root);
    }
    Ok(displaced)
}

fn rewrite_displacement(
    before: &DurablePhysicalRootManifest,
    after: &DurablePhysicalRootManifest,
) -> Option<DisplacedSegmentCharge> {
    if !after.requires_maintenance_protocol() || before.record_count() != after.record_count() {
        return None;
    }
    let before_segment = before.last_inline_segment()?;
    let after_segment = after.last_inline_segment()?;
    if before_segment.segment_id() != after_segment.segment_id()
        || before_segment.generation() == after_segment.generation()
    {
        return None;
    }
    Some(DisplacedSegmentCharge {
        source_root: before.generation(),
        segment_id: before_segment.segment_id().get(),
        generation: before_segment.generation().get(),
    })
}
