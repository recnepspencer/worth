use std::collections::BTreeMap;
use std::num::NonZeroU64;

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DerivedFamilyRootDirectoryBinding, DurablePhysicalRootManifest,
    IndexedThroughBlobPublication, PersistedRecordIdentity, RecordSegmentPageManifestEntry,
    SegmentGenerationCell, SegmentPageKey,
};

use super::inline_segment_plan::InlineSegmentAllocation;
use crate::physical_runtime::record_serving::publication::append_observation::PublicationObservation;

#[path = "prepared_root_projection/released_directory_rebinding.rs"]
mod released_directory_rebinding;

/// Record-serving meaning required to project one successor physical root.
///
/// Durability progression carries this value opaquely. Only record-serving
/// planning may interpret it when the settled group reaches root cutover.
pub(in crate::physical_runtime) struct PreparedPhysicalRootProjection {
    pub(in crate::physical_runtime::record_serving) derived_updates: DerivedRootUpdates,
    pub(in crate::physical_runtime::record_serving) release_head_effect:
        Option<worth_store_physical_format::PersistedReleaseCustodyHeadEffectV1>,
    pub(in crate::physical_runtime::record_serving) arena_reservations:
        Vec<super::super::arena::ArenaReservation>,
    pub(in crate::physical_runtime::record_serving) root_publication_allocation_bytes: NonZeroU64,
    pub(in crate::physical_runtime::record_serving) source_root: DurablePhysicalRootManifest,
    pub(in crate::physical_runtime::record_serving) blob_reuse_source_fence: bool,
    pub(in crate::physical_runtime::record_serving) manifest_capacity_transition:
        crate::physical_runtime::PhysicalManifestCapacityTransition,
    pub(in crate::physical_runtime::record_serving) placement:
        crate::physical_runtime::record_serving::AdmittedRecordPlacementPolicy,
    pub(in crate::physical_runtime::record_serving) records: Vec<PersistedRecordIdentity>,
    /// Identities this publication inserts into the routing tree.
    ///
    /// A rewrite lists existing page records in `records`. Those identities are
    /// already in the source count and must not raise routing height.
    pub(in crate::physical_runtime::record_serving) inserted_records: u64,
    /// Exact identities removed from the fenced source routing root.
    pub(in crate::physical_runtime::record_serving) drop_records:
        std::collections::BTreeSet<PersistedRecordIdentity>,
    pub(in crate::physical_runtime::record_serving) payload_manifests:
        Vec<(worth_store_physical_format::RecordFrameCoordinate, Vec<u8>)>,
    pub(in crate::physical_runtime::record_serving) placements:
        BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    /// Retired slots still physically present in a rewritten inline page.
    /// WAL recovery must validate them, but root publication must not route them.
    pub(in crate::physical_runtime::record_serving) retired_inline_witnesses:
        BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    pub(in crate::physical_runtime::record_serving) segment_updates:
        BTreeMap<SegmentPageKey, RecordSegmentPageManifestEntry>,
    pub(in crate::physical_runtime::record_serving) inline_allocations:
        Vec<InlineSegmentAllocation>,
    pub(in crate::physical_runtime::record_serving) last_inline_record:
        Option<PersistedRecordIdentity>,
    pub(in crate::physical_runtime::record_serving) last_inline_segment:
        Option<SegmentGenerationCell>,
    pub(in crate::physical_runtime::record_serving) observation: PublicationObservation,
    pub(in crate::physical_runtime::record_serving) requires_maintenance_protocol: bool,
}

#[derive(Default, Clone, Copy)]
pub(in crate::physical_runtime::record_serving) struct DerivedRootUpdates {
    pub(in crate::physical_runtime::record_serving) latest_blob_publication:
        Option<IndexedThroughBlobPublication>,
    pub(in crate::physical_runtime::record_serving) latest_blob_quarantine:
        Option<PersistedRecordIdentity>,
    pub(in crate::physical_runtime::record_serving) directory:
        Option<DerivedFamilyRootDirectoryBinding>,
    pub(in crate::physical_runtime::record_serving) expected_previous_directory:
        Option<Option<DerivedFamilyRootDirectoryBinding>>,
    pub(in crate::physical_runtime::record_serving) indexed_through_quarantine:
        Option<Option<PersistedRecordIdentity>>,
    pub(in crate::physical_runtime::record_serving) released_directory_rebinding:
        Option<crate::physical_runtime::record_serving::PreparedReleasedDirectoryRebinding>,
}

impl PreparedPhysicalRootProjection {
    pub(in crate::physical_runtime::record_serving) fn set_release_head_effect(
        &mut self,
        effect: worth_store_physical_format::PersistedReleaseCustodyHeadEffectV1,
    ) -> Option<()> {
        let bytes = effect.framed_bytes()?;
        self.root_publication_allocation_bytes = NonZeroU64::new(
            self.root_publication_allocation_bytes
                .get()
                .checked_add(bytes)?,
        )?;
        self.release_head_effect = Some(effect);
        Some(())
    }

    pub(in crate::physical_runtime) fn recovery_release_head_effect(
        &self,
    ) -> Option<&worth_store_physical_format::PersistedReleaseCustodyHeadEffectV1> {
        self.release_head_effect.as_ref()
    }

    pub(in crate::physical_runtime) const fn recovery_released_directory_rebinding(
        &self,
    ) -> Option<crate::physical_runtime::record_serving::PreparedReleasedDirectoryRebinding> {
        self.derived_updates.released_directory_rebinding
    }

    pub(in crate::physical_runtime) fn set_latest_blob_publication(
        &mut self,
        publication: IndexedThroughBlobPublication,
    ) {
        self.derived_updates.latest_blob_publication = Some(publication);
    }

    pub(in crate::physical_runtime) fn set_latest_blob_quarantine(
        &mut self,
        quarantine: PersistedRecordIdentity,
    ) {
        self.derived_updates.latest_blob_quarantine = Some(quarantine);
    }

    pub(in crate::physical_runtime) fn set_derived_directory(
        &mut self,
        directory: DerivedFamilyRootDirectoryBinding,
        expected_previous: Option<DerivedFamilyRootDirectoryBinding>,
        indexed_through_quarantine: Option<PersistedRecordIdentity>,
        replaced_nodes: &[PersistedRecordIdentity],
    ) {
        self.derived_updates.directory = Some(directory);
        self.derived_updates.expected_previous_directory = Some(expected_previous);
        self.derived_updates.indexed_through_quarantine = Some(indexed_through_quarantine);
        for record in replaced_nodes {
            // An inline-page append can carry forward placements of earlier
            // selected records in the same rewritten page. A protected COW
            // retirement proof makes those records unrouteable in this root;
            // do not reinsert their inherited placement before dropping them.
            // A newly submitted identity must still trip the group conflict.
            if !self.records.contains(record) {
                if let Some(placement @ CurrentPhysicalRecordPlacement::Inline(retired)) =
                    self.placements.remove(record)
                {
                    if self.placements.values().any(|current| {
                        matches!(current, CurrentPhysicalRecordPlacement::Inline(live)
                            if live.page_cell() == retired.page_cell())
                    }) {
                        self.retired_inline_witnesses.insert(*record, placement);
                    }
                }
            }
            self.drop_records.insert(*record);
        }
        if let Some(previous) = expected_previous {
            let record = previous.directory_record();
            if !self.records.contains(&record) {
                if let Some(placement @ CurrentPhysicalRecordPlacement::Inline(retired)) =
                    self.placements.remove(&record)
                {
                    if self.placements.values().any(|current| {
                        matches!(current, CurrentPhysicalRecordPlacement::Inline(live)
                            if live.page_cell() == retired.page_cell())
                    }) {
                        self.retired_inline_witnesses.insert(record, placement);
                    }
                }
            }
            self.drop_records.insert(record);
        }
    }

    pub(in crate::physical_runtime) fn expose_arena_reservations_to_wal(&self) {
        for reservation in &self.arena_reservations {
            reservation.expose_to_wal();
        }
    }

    pub(in crate::physical_runtime) fn arena_wal_proven_no_effect(&self) {
        for reservation in &self.arena_reservations {
            reservation.wal_proven_no_effect();
        }
    }
    pub(in crate::physical_runtime) const fn source_root_generation(&self) -> u64 {
        self.source_root.generation()
    }

    pub(in crate::physical_runtime) const fn manifest_capacity_transition(
        &self,
    ) -> crate::physical_runtime::PhysicalManifestCapacityTransition {
        self.manifest_capacity_transition
    }

    pub(in crate::physical_runtime) const fn recovery_manifest_capacity(&self) -> u16 {
        self.placement.manifest_capacity().get()
    }

    pub(in crate::physical_runtime) fn recovery_placements(
        &self,
    ) -> impl ExactSizeIterator<Item = CurrentPhysicalRecordPlacement> {
        let mut projected = self.placements.values().copied().collect::<Vec<_>>();
        projected.extend(self.retired_inline_witnesses.values().copied());
        projected.sort_unstable_by_key(|placement| placement.record());
        projected.into_iter()
    }

    pub(in crate::physical_runtime) fn recovery_record_identities(
        &self,
    ) -> impl ExactSizeIterator<Item = PersistedRecordIdentity> + '_ {
        self.records.iter().copied()
    }

    pub(in crate::physical_runtime) fn recovery_dropped_record_identities(
        &self,
    ) -> impl ExactSizeIterator<Item = PersistedRecordIdentity> + '_ {
        self.drop_records.iter().copied()
    }

    pub(in crate::physical_runtime) fn recovery_segment_updates(
        &self,
    ) -> impl ExactSizeIterator<Item = RecordSegmentPageManifestEntry> + '_ {
        self.segment_updates.values().copied()
    }

    pub(in crate::physical_runtime) fn recovery_payload_manifests(
        &self,
    ) -> impl ExactSizeIterator<Item = &(worth_store_physical_format::RecordFrameCoordinate, Vec<u8>)>
    {
        self.payload_manifests.iter()
    }

    pub(in crate::physical_runtime) const fn root_publication_allocation_bytes(
        &self,
    ) -> NonZeroU64 {
        self.root_publication_allocation_bytes
    }

    /// Bytes every publication retains beside its data frames.
    ///
    /// Fixed root, free-space header, selector, and catalog bytes are included.
    /// Record, segment, and free-space routing are charged as full trees of
    /// full-capacity blocks. Payload manifests are not added again: the WAL
    /// frame already carries them, and reopen can price only what it reads.
    pub(in crate::physical_runtime) fn retained_publication_metadata_bytes(&self) -> u64 {
        let entries = self
            .source_root
            .record_count()
            .saturating_add(self.inserted_records)
            .max(1);
        let head_bytes = self.release_head_effect.as_ref().map_or(0, |effect| {
            effect.node_writes().iter().fold(0_u64, |bytes, write| {
                bytes.saturating_add(write.frame().len() as u64)
            })
        });
        publication_metadata_reservation(self.placement.manifest_capacity().get(), entries)
            .saturating_add(head_bytes)
    }

    pub(in crate::physical_runtime) fn recovery_inline_allocations(
        &self,
    ) -> impl ExactSizeIterator<Item = super::inline_segment_plan::InlineSegmentAllocation> + '_
    {
        self.inline_allocations.iter().copied()
    }

    pub(in crate::physical_runtime) const fn recovery_last_inline_record(
        &self,
    ) -> Option<PersistedRecordIdentity> {
        self.last_inline_record
    }

    pub(in crate::physical_runtime) const fn recovery_last_inline_segment(
        &self,
    ) -> Option<SegmentGenerationCell> {
        self.last_inline_segment
    }

    pub(in crate::physical_runtime::record_serving) fn completion_projection(
        &self,
    ) -> crate::physical_runtime::record_serving::PreparedRecordCompletionProjection {
        crate::physical_runtime::record_serving::PreparedRecordCompletionProjection::new(
            &self.records,
            self.observation,
        )
    }

    pub(in crate::physical_runtime::record_serving) fn settle_data_observation(
        &mut self,
        transfer_count: u64,
    ) {
        self.observation.settle_data_transfers(transfer_count);
    }

    pub(in crate::physical_runtime::record_serving) fn into_payload_plan(
        self,
    ) -> super::prepared_payload::PreparedRecordPayloadPlan {
        super::prepared_payload::PreparedRecordPayloadPlan {
            derived_updates: self.derived_updates,
            release_head_effect: self.release_head_effect,
            arena_reservations: self.arena_reservations,
            source_root: self.source_root,
            blob_reuse_source_fence: self.blob_reuse_source_fence,
            manifest_capacity_transition: self.manifest_capacity_transition,
            placement: self.placement,
            records: self.records,
            drop_records: self.drop_records,
            data: Vec::new(),
            payload_manifests: self.payload_manifests,
            placements: self.placements,
            segment_updates: self.segment_updates,
            inline_allocations: self.inline_allocations,
            last_inline_record: self.last_inline_record,
            last_inline_segment: self.last_inline_segment,
            observation: self.observation,
            requires_maintenance_protocol: self.requires_maintenance_protocol,
        }
    }
}

/// Conservative admission ceiling, never an observation of bytes actually kept.
pub(in crate::physical_runtime) fn publication_metadata_reservation(
    capacity: u16,
    entries: u64,
) -> u64 {
    canonical_publication_metadata_bytes().saturating_add(routing_publication_bound(
        u64::from(capacity),
        entries.max(1),
    ))
}

fn routing_publication_bound(capacity: u64, entries: u64) -> u64 {
    let (leaves, branches) = routing_tree_shape(entries, capacity);
    let record = tree_bytes(leaves, branches, capacity, 88, 72);
    let membership = tree_bytes(leaves, branches, capacity, 40, 56);
    let free_space = tree_bytes(leaves, branches, capacity, 40, 72);
    record.saturating_add(membership).saturating_add(free_space)
}

/// Leaf count and branch count of one full routing tree.
fn routing_tree_shape(entries: u64, capacity: u64) -> (u64, u64) {
    if entries == 0 || capacity < 2 {
        return (0, 0);
    }
    let leaves = entries.div_ceil(capacity);
    let mut branches = 0_u64;
    let mut nodes = leaves;
    while nodes > 1 {
        nodes = nodes.div_ceil(capacity);
        branches = branches.saturating_add(nodes);
    }
    (leaves, branches)
}

fn tree_bytes(
    leaves: u64,
    branches: u64,
    capacity: u64,
    leaf_entry: u64,
    branch_entry: u64,
) -> u64 {
    let header = worth_store_physical_format::DURABLE_FRAME_HEADER_BYTES as u64;
    let block = |entry: u64| {
        header
            .saturating_add(40)
            .saturating_add(capacity.saturating_mul(entry))
    };
    leaves
        .saturating_mul(block(leaf_entry))
        .saturating_add(branches.saturating_mul(block(branch_entry)))
}

#[cfg(test)]
mod routing_bound_tests {
    use super::routing_publication_bound;

    #[test]
    fn wide_publication_reserves_every_routing_node() {
        // Capacity 2 and 32 records emit 16 leaves and 15 branches.
        // Full record leaves are 264 bytes and branches 232, so record routing
        // alone is 7704. Segment and free-space trees share that shape.
        assert_eq!(routing_publication_bound(2, 32), 7_704 + 5_688 + 6_168);
    }
}

fn canonical_publication_metadata_bytes() -> u64 {
    let header = worth_store_physical_format::DURABLE_FRAME_HEADER_BYTES as u64;
    // The head-bearing schema includes the tier anchor and custody-tree
    // reference. Charge the largest root any admitted publication may select.
    let root_manifest = header.saturating_add(680);
    let free_space = header.saturating_add(168);
    let selectors = 2 * worth_store_physical_format::ROOT_SELECTOR_BYTES as u64;
    let catalog = worth_store_physical_format::BOOTSTRAP_CATALOG_BYTES as u64;
    root_manifest
        .saturating_add(free_space)
        .saturating_add(selectors)
        .saturating_add(catalog)
}
