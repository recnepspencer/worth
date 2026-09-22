use std::collections::BTreeMap;
use std::num::NonZeroU64;

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PersistedRecordIdentity,
    RecordArtifactFile, RecordSegmentPageManifestEntry, SegmentGenerationCell, SegmentPageKey,
};

use super::inline_segment_plan::InlineSegmentAllocation;
use crate::physical_runtime::record_serving::publication::append_observation::PublicationObservation;

/// Record-serving meaning required to project one successor physical root.
///
/// Durability progression carries this value opaquely. Only record-serving
/// planning may interpret it when the settled group reaches root cutover.
pub(in crate::physical_runtime) struct PreparedPhysicalRootProjection {
    pub(in crate::physical_runtime::record_serving) root_publication_allocation_bytes: NonZeroU64,
    pub(in crate::physical_runtime::record_serving) source_root: DurablePhysicalRootManifest,
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
    pub(in crate::physical_runtime::record_serving) payload_manifests:
        Vec<(RecordArtifactFile, Vec<u8>)>,
    pub(in crate::physical_runtime::record_serving) placements:
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

impl PreparedPhysicalRootProjection {
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
    ) -> impl ExactSizeIterator<Item = CurrentPhysicalRecordPlacement> + '_ {
        self.placements.values().copied()
    }

    pub(in crate::physical_runtime) fn recovery_record_identities(
        &self,
    ) -> impl ExactSizeIterator<Item = PersistedRecordIdentity> + '_ {
        self.records.iter().copied()
    }

    pub(in crate::physical_runtime) fn recovery_segment_updates(
        &self,
    ) -> impl ExactSizeIterator<Item = RecordSegmentPageManifestEntry> + '_ {
        self.segment_updates.values().copied()
    }

    pub(in crate::physical_runtime) fn recovery_payload_manifests(
        &self,
    ) -> impl ExactSizeIterator<Item = &(RecordArtifactFile, Vec<u8>)> {
        self.payload_manifests.iter()
    }

    pub(in crate::physical_runtime) const fn root_publication_allocation_bytes(
        &self,
    ) -> NonZeroU64 {
        self.root_publication_allocation_bytes
    }

    /// Bytes every publication retains beside its data frames.
    ///
    /// Fixed root, free-space header, selector, and catalog bytes are included,
    /// plus payload manifests already planned. Record, segment, and free-space
    /// routing are charged as full trees of full-capacity blocks.
    pub(in crate::physical_runtime) fn retained_publication_metadata_bytes(&self) -> u64 {
        let manifests = self.payload_manifests.iter().fold(0_u64, |total, (_, bytes)| {
            total.saturating_add(bytes.len() as u64)
        });
        manifests
            .saturating_add(canonical_publication_metadata_bytes())
            .saturating_add(self.routing_publication_bound())
    }

    fn routing_publication_bound(&self) -> u64 {
        let entries = self
            .source_root
            .record_count()
            .saturating_add(self.inserted_records)
            .max(1);
        routing_publication_bound(u64::from(self.placement.manifest_capacity().get()), entries)
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
        effect_count: usize,
    ) {
        self.observation.settle_data_effects(effect_count);
    }

    pub(in crate::physical_runtime::record_serving) fn into_payload_plan(
        self,
    ) -> super::prepared_payload::PreparedRecordPayloadPlan {
        super::prepared_payload::PreparedRecordPayloadPlan {
            source_root: self.source_root,
            manifest_capacity_transition: self.manifest_capacity_transition,
            placement: self.placement,
            records: self.records,
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

pub(in crate::physical_runtime) fn sealed_publication_overhead(
    root: &worth_store_physical_format::DurablePhysicalRootManifest,
) -> u64 {
    canonical_publication_metadata_bytes().saturating_add(routing_publication_bound(
        u64::from(root.node_capacity()),
        root.record_count().max(1),
    ))
}

fn routing_publication_bound(capacity: u64, entries: u64) -> u64 {
    let (leaves, branches) = routing_tree_shape(entries, capacity);
    let record = tree_bytes(leaves, branches, capacity, 88, 72);
    let membership = tree_bytes(leaves, branches, capacity, 40, 56);
    record.saturating_add(membership).saturating_add(membership)
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

fn tree_bytes(leaves: u64, branches: u64, capacity: u64, leaf_entry: u64, branch_entry: u64) -> u64 {
    let header = worth_store_physical_format::DURABLE_FRAME_HEADER_BYTES as u64;
    let block = |entry: u64| header.saturating_add(40).saturating_add(capacity.saturating_mul(entry));
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
        assert_eq!(routing_publication_bound(2, 32), 7_704 + 5_688 + 5_688);
    }
}

fn canonical_publication_metadata_bytes() -> u64 {
    let header = worth_store_physical_format::DURABLE_FRAME_HEADER_BYTES as u64;
    let root_manifest = header.saturating_add(320);
    let free_space = header.saturating_add(128);
    let selectors = 2 * worth_store_physical_format::ROOT_SELECTOR_BYTES as u64;
    let catalog = worth_store_physical_format::BOOTSTRAP_CATALOG_BYTES as u64;
    root_manifest
        .saturating_add(free_space)
        .saturating_add(selectors)
        .saturating_add(catalog)
}
