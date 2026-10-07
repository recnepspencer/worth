use std::{collections::BTreeSet, num::NonZeroU64};

use worth_proof::NonEmpty;
use worth_store_physical_format::{PersistedRecordIdentity, RecordFrameCoordinate};

use super::{prepared_payload::PreparedRecordPayloadPlan, PreparedPhysicalRootProjection};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettledRootProjectionMergeDenial {
    SourceRootMismatch,
    PlacementPolicyMismatch,
    ManifestCapacityTransitionMismatch,
    DuplicateRecord,
    DuplicatePayloadManifest,
    OverlappingPayloadManifest,
    DuplicatePlacement,
    DuplicateSegmentUpdate,
    DuplicateDerivedDirectoryUpdate,
    DuplicateBlobPublicationUpdate,
    DuplicateBlobQuarantineUpdate,
    DuplicateDroppedRecord,
    DropConflictsWithPlacement,
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
        if projection.derived_updates.latest_blob_publication.is_some() {
            merged.derived_updates.latest_blob_publication =
                projection.derived_updates.latest_blob_publication;
        }
        if projection.derived_updates.latest_blob_quarantine.is_some() {
            merged.derived_updates.latest_blob_quarantine =
                projection.derived_updates.latest_blob_quarantine;
        }
        if projection.derived_updates.directory.is_some() {
            merged.derived_updates.directory = projection.derived_updates.directory;
            merged.derived_updates.expected_previous_directory =
                projection.derived_updates.expected_previous_directory;
            merged.derived_updates.indexed_through_quarantine =
                projection.derived_updates.indexed_through_quarantine;
            merged.derived_updates.released_directory_rebinding =
                projection.derived_updates.released_directory_rebinding;
        }
        merged
            .arena_reservations
            .extend(projection.arena_reservations);
        merged.records.extend(projection.records);
        merged.drop_records.extend(projection.drop_records);
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
        merged.blob_reuse_source_fence |= projection.blob_reuse_source_fence;
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
    let mut payload_artifacts = BTreeSet::<RecordFrameCoordinate>::new();
    let mut placements = BTreeSet::new();
    let mut drops = BTreeSet::new();
    let mut segment_updates = BTreeSet::new();
    let mut saw_directory_update = false;
    let mut saw_blob_publication_update = false;
    let mut saw_blob_quarantine_update = false;
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
        admit_unique_directory_update(
            &mut saw_directory_update,
            projection.derived_updates.directory.is_some(),
        )?;
        admit_unique_blob_publication_update(
            &mut saw_blob_publication_update,
            projection.derived_updates.latest_blob_publication.is_some(),
        )?;
        admit_unique_blob_quarantine_update(
            &mut saw_blob_quarantine_update,
            projection.derived_updates.latest_blob_quarantine.is_some(),
        )?;
        allocation_bytes = allocation_bytes
            .checked_add(projection.root_publication_allocation_bytes().get())
            .ok_or(SettledRootProjectionMergeDenial::AllocationBudgetOverflow)?;
        for record in &projection.records {
            if !records.insert(*record) {
                return Err(SettledRootProjectionMergeDenial::DuplicateRecord);
            }
        }
        for (artifact, _) in &projection.payload_manifests {
            insert_payload_range(&mut payload_artifacts, *artifact)?;
        }
        for record in projection.placements.keys() {
            if !placements.insert(*record) {
                return Err(SettledRootProjectionMergeDenial::DuplicatePlacement);
            }
        }
        for record in &projection.drop_records {
            if !drops.insert(*record) {
                return Err(SettledRootProjectionMergeDenial::DuplicateDroppedRecord);
            }
        }
        for page in projection.segment_updates.keys() {
            if !segment_updates.insert(*page) {
                return Err(SettledRootProjectionMergeDenial::DuplicateSegmentUpdate);
            }
        }
    }
    if drops.iter().any(|record| placements.contains(record)) {
        return Err(SettledRootProjectionMergeDenial::DropConflictsWithPlacement);
    }
    NonZeroU64::new(allocation_bytes)
        .ok_or(SettledRootProjectionMergeDenial::AllocationBudgetOverflow)
}

fn admit_unique_directory_update(
    saw_directory_update: &mut bool,
    member_updates_directory: bool,
) -> Result<(), SettledRootProjectionMergeDenial> {
    if member_updates_directory && std::mem::replace(saw_directory_update, true) {
        return Err(SettledRootProjectionMergeDenial::DuplicateDerivedDirectoryUpdate);
    }
    Ok(())
}

fn admit_unique_blob_publication_update(
    saw_blob_publication_update: &mut bool,
    member_publishes_blob: bool,
) -> Result<(), SettledRootProjectionMergeDenial> {
    if member_publishes_blob && std::mem::replace(saw_blob_publication_update, true) {
        return Err(SettledRootProjectionMergeDenial::DuplicateBlobPublicationUpdate);
    }
    Ok(())
}

fn admit_unique_blob_quarantine_update(
    saw_blob_quarantine_update: &mut bool,
    member_updates_quarantine: bool,
) -> Result<(), SettledRootProjectionMergeDenial> {
    if member_updates_quarantine && std::mem::replace(saw_blob_quarantine_update, true) {
        return Err(SettledRootProjectionMergeDenial::DuplicateBlobQuarantineUpdate);
    }
    Ok(())
}

fn insert_payload_range(
    ranges: &mut BTreeSet<RecordFrameCoordinate>,
    coordinate: RecordFrameCoordinate,
) -> Result<(), SettledRootProjectionMergeDenial> {
    if ranges.contains(&coordinate) {
        return Err(SettledRootProjectionMergeDenial::DuplicatePayloadManifest);
    }
    let overlaps = |other: &RecordFrameCoordinate| {
        other.artifact() == coordinate.artifact()
            && other.offset() < coordinate.offset() + u64::from(coordinate.length())
            && coordinate.offset() < other.offset() + u64::from(other.length())
    };
    if ranges.range(..coordinate).next_back().is_some_and(overlaps)
        || ranges.range(coordinate..).next().is_some_and(overlaps)
    {
        return Err(SettledRootProjectionMergeDenial::OverlappingPayloadManifest);
    }
    ranges.insert(coordinate);
    Ok(())
}

#[cfg(test)]
mod payload_range_tests {
    use super::*;
    use worth_store_physical_format::RecordArtifactFile;

    #[test]
    fn shared_arena_accepts_disjoint_manifests_but_rejects_partial_overlap() {
        let range = |arena, offset, length| {
            RecordFrameCoordinate::new(RecordArtifactFile::ExtentArena { arena }, offset, length)
                .unwrap()
        };
        let mut ranges = BTreeSet::new();
        insert_payload_range(&mut ranges, range(1, 4096, 104)).unwrap();
        insert_payload_range(&mut ranges, range(1, 8192, 104)).unwrap();
        insert_payload_range(&mut ranges, range(2, 4096, 104)).unwrap();
        assert_eq!(
            insert_payload_range(&mut ranges, range(1, 4096, 104)),
            Err(SettledRootProjectionMergeDenial::DuplicatePayloadManifest)
        );
        assert_eq!(
            insert_payload_range(&mut ranges, range(1, 4097, 104)),
            Err(SettledRootProjectionMergeDenial::OverlappingPayloadManifest)
        );
        assert_eq!(
            insert_payload_range(&mut ranges, range(1, 4080, 104)),
            Err(SettledRootProjectionMergeDenial::OverlappingPayloadManifest)
        );
    }
}

#[cfg(test)]
mod directory_group_tests {
    use super::*;

    #[test]
    fn two_classified_directory_members_in_one_group_are_denied() {
        let mut saw_directory_update = false;
        for classified in [false, true, false] {
            admit_unique_directory_update(&mut saw_directory_update, classified).unwrap();
        }
        assert_eq!(
            admit_unique_directory_update(&mut saw_directory_update, true),
            Err(SettledRootProjectionMergeDenial::DuplicateDerivedDirectoryUpdate)
        );
    }

    #[test]
    fn two_blob_publication_members_in_one_group_are_denied_before_last_wins() {
        let mut saw_blob_publication_update = false;
        admit_unique_blob_publication_update(&mut saw_blob_publication_update, true).unwrap();
        assert_eq!(
            admit_unique_blob_publication_update(&mut saw_blob_publication_update, true),
            Err(SettledRootProjectionMergeDenial::DuplicateBlobPublicationUpdate)
        );
    }

    #[test]
    fn two_quarantine_members_in_one_group_are_denied_before_last_wins() {
        let mut saw = false;
        admit_unique_blob_quarantine_update(&mut saw, true).unwrap();
        assert_eq!(
            admit_unique_blob_quarantine_update(&mut saw, true),
            Err(SettledRootProjectionMergeDenial::DuplicateBlobQuarantineUpdate)
        );
    }
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
