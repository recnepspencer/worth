use std::{collections::BTreeSet, num::NonZeroU64};

use worth_proof::NonEmpty;
use worth_store_physical_format::{PersistedRecordIdentity, RecordArtifactFile};

use super::{prepared_payload::PreparedRecordPayloadPlan, PreparedPhysicalRootProjection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettledRootProjectionMergeDenial {
    SourceRootMismatch,
    PlacementPolicyMismatch,
    ManifestCapacityTransitionMismatch,
    DuplicateRecord,
    DuplicatePayloadManifest,
    DuplicatePlacement,
    DuplicateSegmentUpdate,
    AllocationBudgetOverflow,
}

pub(in crate::physical_runtime) struct MergedSettledRootProjection {
    prepared: PreparedRecordPayloadPlan,
    allocation_bytes: NonZeroU64,
}

pub(in crate::physical_runtime) struct RejectedSettledRootProjections {
    cause: SettledRootProjectionMergeDenial,
}

impl MergedSettledRootProjection {
    pub(in crate::physical_runtime::record_serving) fn into_parts(
        self,
    ) -> (PreparedRecordPayloadPlan, NonZeroU64) {
        (self.prepared, self.allocation_bytes)
    }
}

impl RejectedSettledRootProjections {
    pub(in crate::physical_runtime) const fn cause(&self) -> SettledRootProjectionMergeDenial {
        self.cause
    }
}

pub(in crate::physical_runtime::record_serving) fn merge_settled_root_projections(
    projections: NonEmpty<PreparedPhysicalRootProjection>,
) -> Result<
    MergedSettledRootProjection,
    (
        NonEmpty<PreparedPhysicalRootProjection>,
        RejectedSettledRootProjections,
    ),
> {
    let allocation_bytes = match validate(&projections) {
        Ok(allocation_bytes) => allocation_bytes,
        Err(cause) => return Err((projections, RejectedSettledRootProjections { cause })),
    };
    let mut projections = projections.into_vec().into_iter();
    let first = projections
        .next()
        .expect("NonEmpty settled root projections contain one member");
    let mut merged = first.into_payload_plan();
    for projection in projections {
        merged.records.extend(projection.records);
        merged
            .payload_manifests
            .extend(projection.payload_manifests);
        merged.placements.extend(projection.placements);
        merged.segment_updates.extend(projection.segment_updates);
        merged
            .inline_allocations
            .extend(projection.inline_allocations);
        if projection.last_inline_record.is_some() {
            merged.last_inline_record = projection.last_inline_record;
            merged.last_inline_segment = projection.last_inline_segment;
        }
        merged.requires_maintenance_protocol |= projection.requires_maintenance_protocol;
        merge_observation(&mut merged.observation, projection.observation);
    }
    Ok(MergedSettledRootProjection {
        prepared: merged,
        allocation_bytes,
    })
}

fn validate(
    projections: &NonEmpty<PreparedPhysicalRootProjection>,
) -> Result<NonZeroU64, SettledRootProjectionMergeDenial> {
    let first = projections.first();
    let mut allocation_bytes = 0_u64;
    let mut records = BTreeSet::<PersistedRecordIdentity>::new();
    let mut payload_artifacts = BTreeSet::<RecordArtifactFile>::new();
    let mut placements = BTreeSet::new();
    let mut segment_updates = BTreeSet::new();
    for projection in projections.as_slice() {
        if projection.source_root != first.source_root {
            return Err(SettledRootProjectionMergeDenial::SourceRootMismatch);
        }
        if projection.placement != first.placement {
            return Err(SettledRootProjectionMergeDenial::PlacementPolicyMismatch);
        }
        if projection.manifest_capacity_transition != first.manifest_capacity_transition {
            return Err(SettledRootProjectionMergeDenial::ManifestCapacityTransitionMismatch);
        }
        allocation_bytes = allocation_bytes
            .checked_add(projection.root_publication_allocation_bytes().get())
            .ok_or(SettledRootProjectionMergeDenial::AllocationBudgetOverflow)?;
        for record in &projection.records {
            if !records.insert(*record) {
                return Err(SettledRootProjectionMergeDenial::DuplicateRecord);
            }
        }
        for (artifact, _) in &projection.payload_manifests {
            if !payload_artifacts.insert(*artifact) {
                return Err(SettledRootProjectionMergeDenial::DuplicatePayloadManifest);
            }
        }
        for record in projection.placements.keys() {
            if !placements.insert(*record) {
                return Err(SettledRootProjectionMergeDenial::DuplicatePlacement);
            }
        }
        for page in projection.segment_updates.keys() {
            if !segment_updates.insert(*page) {
                return Err(SettledRootProjectionMergeDenial::DuplicateSegmentUpdate);
            }
        }
    }
    NonZeroU64::new(allocation_bytes)
        .ok_or(SettledRootProjectionMergeDenial::AllocationBudgetOverflow)
}

fn merge_observation(
    merged: &mut super::super::publication::append_observation::PublicationObservation,
    incoming: super::super::publication::append_observation::PublicationObservation,
) {
    merged.records = merged.records.saturating_add(incoming.records);
    merged.logical_bytes = merged.logical_bytes.saturating_add(incoming.logical_bytes);
    merged.completed_bytes = merged
        .completed_bytes
        .saturating_add(incoming.completed_bytes);
    merged.segment_artifacts = merged
        .segment_artifacts
        .saturating_add(incoming.segment_artifacts);
    merged.extent_artifacts = merged
        .extent_artifacts
        .saturating_add(incoming.extent_artifacts);
    merged.transfer_count = merged
        .transfer_count
        .saturating_add(incoming.transfer_count);
    merged.peak_transfer_width = merged.peak_transfer_width.max(incoming.peak_transfer_width);
    merged.explicit_copy_count = merged
        .explicit_copy_count
        .saturating_add(incoming.explicit_copy_count);
    merged.copied_bytes = merged.copied_bytes.saturating_add(incoming.copied_bytes);
    merged.peak_scratch_bytes = merged.peak_scratch_bytes.max(incoming.peak_scratch_bytes);
    merged.manifest_blocks_read = merged
        .manifest_blocks_read
        .saturating_add(incoming.manifest_blocks_read);
    merged.manifest_comparisons = merged
        .manifest_comparisons
        .saturating_add(incoming.manifest_comparisons);
    merged.manifest_bytes_read = merged
        .manifest_bytes_read
        .saturating_add(incoming.manifest_bytes_read);
}
