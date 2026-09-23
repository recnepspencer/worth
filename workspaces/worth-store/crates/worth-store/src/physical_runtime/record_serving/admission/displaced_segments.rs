use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalSegmentId, RecordArtifactFile,
};

use super::super::access::manifest_routing::{
    ManifestDiscoveryCounterSnapshot, ManifestLookupFailure,
};
use super::super::access::segment_membership::SegmentMembershipReader;
use super::super::residency::serving_artifacts::ServingRecordArtifacts;
use super::bootstrap::{
    backend_before_effect, BootstrapTransitionFailure, DisplacedSegmentCharge,
    RecordBootstrapDenial,
};

/// Rewrite steps in the publication chain whose source segment file remains
/// although the current root reads no frame from it.
///
/// An append changes the record count, so its predecessor stays out of this
/// charge. A rewrite keeps the count and advances the same segment generation.
/// A span rewrite can leave live frames before the span in its source file,
/// so only the current membership decides whether the source is displaced.
/// Retirement removes the source file; the charge ends with the file.
pub(super) fn retained_displaced_segments(
    prior: &[DurablePhysicalRootManifest],
    current: &DurablePhysicalRootManifest,
    artifacts: &ServingRecordArtifacts,
    membership: &SegmentMembershipReader<'_>,
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
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
                if present && unread_by_current_root(membership, allocation, &charge)? {
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

/// Reads only the membership blocks holding the charged segment.
fn unread_by_current_root(
    membership: &SegmentMembershipReader<'_>,
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    charge: &DisplacedSegmentCharge,
) -> Result<bool, BootstrapTransitionFailure> {
    let segment =
        PhysicalSegmentId::from_raw(charge.segment_id).map_err(|_| current_root_damaged())?;
    let mut discovery = ManifestDiscoveryCounterSnapshot::default();
    let entries = membership
        .segment_entries(allocation, segment, &mut discovery)
        .map_err(membership_failure)?;
    Ok(entries
        .iter()
        .all(|entry| entry.data_generation() != charge.generation))
}

fn membership_failure(failure: ManifestLookupFailure) -> BootstrapTransitionFailure {
    match failure {
        ManifestLookupFailure::Backend(failure) => backend_before_effect(failure),
        ManifestLookupFailure::Residency(denial) => {
            BootstrapTransitionFailure::Denied(RecordBootstrapDenial::from_residency(denial))
        }
        ManifestLookupFailure::Frame(_)
        | ManifestLookupFailure::ResidentAdmission(_)
        | ManifestLookupFailure::Damaged => current_root_damaged(),
    }
}

fn current_root_damaged() -> BootstrapTransitionFailure {
    BootstrapTransitionFailure::Denied(RecordBootstrapDenial::CurrentRootDamaged)
}
