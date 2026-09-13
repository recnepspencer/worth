use std::collections::{BTreeMap, BTreeSet, VecDeque};

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, ManifestBlockReference, PhysicalRootRoutingBlock,
};

use super::PhysicalRootSourceCandidate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalManifestBlockProjection {
    reference: ManifestBlockReference,
    block: PhysicalRootRoutingBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedPhysicalPageFacts {
    root_generation: u64,
    manifest_block_count: u64,
    distinct_pages_and_extents: u64,
    routing_blocks: Vec<ManifestBlockReference>,
    routing_topology: Vec<(ManifestBlockReference, PhysicalRootRoutingBlock)>,
    placements: Vec<CurrentPhysicalRecordPlacement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalPageFactDenial {
    DuplicateManifestBlock,
    MissingManifestBlock,
    UnexpectedManifestBlock,
    ManifestEntryLimit,
    ManifestReferenceIdentityMismatch,
    TreeIdentityMismatch,
    DuplicateRecord,
    RecordCountMismatch,
    DistinctPageOrExtentLimit,
}

impl PhysicalManifestBlockProjection {
    /// Carries a block projection into source-precedence evaluation.
    ///
    /// This value is descriptive, not integrity authority. Its reference
    /// identity is checked again before owner facts are formed.
    pub fn from_projected_block(
        reference: ManifestBlockReference,
        block: PhysicalRootRoutingBlock,
    ) -> Self {
        Self { reference, block }
    }
}

impl SelectedPhysicalPageFacts {
    pub const fn root_generation(&self) -> u64 {
        self.root_generation
    }

    pub const fn manifest_block_count(&self) -> u64 {
        self.manifest_block_count
    }

    pub const fn distinct_pages_and_extents(&self) -> u64 {
        self.distinct_pages_and_extents
    }

    pub fn placements(&self) -> &[CurrentPhysicalRecordPlacement] {
        &self.placements
    }

    pub fn routing_blocks(&self) -> &[ManifestBlockReference] {
        &self.routing_blocks
    }

    pub fn routing_topology(&self) -> &[(ManifestBlockReference, PhysicalRootRoutingBlock)] {
        &self.routing_topology
    }
}

pub fn admit_physical_page_facts(
    root: &PhysicalRootSourceCandidate,
    blocks: Vec<PhysicalManifestBlockProjection>,
    maximum_manifest_entries: u64,
    maximum_distinct_pages_and_extents: u64,
) -> Result<SelectedPhysicalPageFacts, PhysicalPageFactDenial> {
    let mut candidates = BTreeMap::new();
    for candidate in blocks {
        let key = reference_key(candidate.reference);
        if candidates.insert(key, candidate).is_some() {
            return Err(PhysicalPageFactDenial::DuplicateManifestBlock);
        }
    }
    let manifest = root.manifest();
    let mut pending = manifest.routing_root().into_iter().collect::<VecDeque<_>>();
    let mut visited = BTreeSet::new();
    let mut routing_blocks = Vec::new();
    let mut routing_topology = Vec::new();
    let mut placements = Vec::new();
    let mut distinct_pages_and_extents = BTreeSet::new();
    while let Some(reference) = pending.pop_front() {
        if !visited.insert(reference_key(reference)) {
            return Err(PhysicalPageFactDenial::DuplicateManifestBlock);
        }
        routing_blocks.push(reference);
        let candidate = candidates
            .remove(&reference_key(reference))
            .ok_or(PhysicalPageFactDenial::MissingManifestBlock)?;
        let block = candidate.block;
        if block.reference(reference.checksum()) != reference {
            return Err(PhysicalPageFactDenial::ManifestReferenceIdentityMismatch);
        }
        if block.tree_identity() != manifest.tree_identity() {
            return Err(PhysicalPageFactDenial::TreeIdentityMismatch);
        }
        routing_topology.push((reference, block.clone()));
        match block {
            PhysicalRootRoutingBlock::Leaf { entries, .. } => {
                let next_entry_count = placements
                    .len()
                    .checked_add(entries.len())
                    .ok_or(PhysicalPageFactDenial::ManifestEntryLimit)?;
                if next_entry_count as u64 > maximum_manifest_entries {
                    return Err(PhysicalPageFactDenial::ManifestEntryLimit);
                }
                for placement in &entries {
                    let key = page_or_extent_key(placement);
                    if !distinct_pages_and_extents.contains(&key) {
                        if distinct_pages_and_extents.len() as u64
                            == maximum_distinct_pages_and_extents
                        {
                            return Err(PhysicalPageFactDenial::DistinctPageOrExtentLimit);
                        }
                        distinct_pages_and_extents.insert(key);
                    }
                }
                placements.extend(entries);
            }
            PhysicalRootRoutingBlock::Branch { children, .. } => pending.extend(children),
        }
    }
    if !candidates.is_empty() {
        return Err(PhysicalPageFactDenial::UnexpectedManifestBlock);
    }
    placements.sort_unstable_by_key(|placement| placement.record());
    if placements
        .windows(2)
        .any(|pair| pair[0].record() == pair[1].record())
    {
        return Err(PhysicalPageFactDenial::DuplicateRecord);
    }
    if placements.len() as u64 != manifest.record_count() {
        return Err(PhysicalPageFactDenial::RecordCountMismatch);
    }
    let distinct_pages_and_extents = distinct_pages_and_extents.len() as u64;
    Ok(SelectedPhysicalPageFacts {
        root_generation: manifest.generation(),
        manifest_block_count: routing_blocks.len() as u64,
        distinct_pages_and_extents,
        routing_blocks,
        routing_topology,
        placements,
    })
}

const fn reference_key(reference: ManifestBlockReference) -> (u64, u64) {
    (reference.generation(), reference.block())
}

fn page_or_extent_key(placement: &CurrentPhysicalRecordPlacement) -> (u8, u64, u64, u64) {
    match placement {
        CurrentPhysicalRecordPlacement::Inline(value) => (
            0,
            value.segment().get(),
            value.page().get(),
            value.page_generation(),
        ),
        CurrentPhysicalRecordPlacement::Extent(value) => {
            (1, 0, value.extent().get(), value.extent_generation())
        }
    }
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::{
        durable_artifact_checksum,
        store_namespace::{
            ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord,
            StoreNamespaceVersion,
        },
        DurableExtentRecordPlacement, DurablePhysicalRootManifest, DurableRootSelector,
        FreeSpaceBlockReference, FreeSpaceKey, PersistedRecordIdentity, PhysicalExtentId,
        PhysicalGeneration, PhysicalGenerationAuthority, PhysicalRecordFormatDeclaration,
        RecordAllocationClass, RootSelectorIdentity, RootSelectorRole,
    };

    use super::*;
    use crate::{observe_structured_physical_root_candidate, PhysicalRootSlotObservation};

    #[test]
    fn exact_manifest_addressed_fact_set_is_admitted() {
        let (root, block) = root_and_block(1);
        let facts = admit_physical_page_facts(&root, vec![block], 1, 1).unwrap();
        assert_eq!(facts.root_generation(), 1);
        assert_eq!(facts.manifest_block_count(), 1);
        assert_eq!(facts.placements().len(), 1);
        assert_eq!(facts.distinct_pages_and_extents(), 1);
    }

    #[test]
    fn missing_extra_and_over_budget_blocks_cannot_form_page_facts() {
        let (root, block) = root_and_block(1);
        assert_eq!(
            admit_physical_page_facts(&root, Vec::new(), 1, 1),
            Err(PhysicalPageFactDenial::MissingManifestBlock)
        );
        assert_eq!(
            admit_physical_page_facts(&root, vec![block.clone(), block.clone()], 2, 1),
            Err(PhysicalPageFactDenial::DuplicateManifestBlock)
        );
        assert_eq!(
            admit_physical_page_facts(&root, vec![block], 0, 1),
            Err(PhysicalPageFactDenial::ManifestEntryLimit)
        );
    }

    #[test]
    fn selected_root_record_count_and_distinct_fact_limit_are_independent() {
        let (overstated_root, block) = root_and_block(2);
        assert_eq!(
            admit_physical_page_facts(&overstated_root, vec![block], 1, 1),
            Err(PhysicalPageFactDenial::RecordCountMismatch)
        );
        let (root, block) = root_and_block(1);
        assert_eq!(
            admit_physical_page_facts(&root, vec![block], 1, 0),
            Err(PhysicalPageFactDenial::DistinctPageOrExtentLimit)
        );
    }

    #[test]
    fn projected_block_range_cannot_substitute_for_its_reference() {
        let (root, candidate) = root_and_block(1);
        let replacement = PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![placement(2)], 4).unwrap();
        let substituted =
            PhysicalManifestBlockProjection::from_projected_block(candidate.reference, replacement);
        assert_eq!(
            admit_physical_page_facts(&root, vec![substituted], 1, 1),
            Err(PhysicalPageFactDenial::ManifestReferenceIdentityMismatch)
        );
    }

    #[test]
    fn aggregate_entries_across_multiple_leaf_blocks_are_bounded() {
        let (root, blocks) = branched_root_and_blocks();
        assert_eq!(
            admit_physical_page_facts(&root, blocks.clone(), 2, 3),
            Err(PhysicalPageFactDenial::ManifestEntryLimit)
        );
        let facts = admit_physical_page_facts(&root, blocks, 3, 3).unwrap();
        assert_eq!(facts.manifest_block_count(), 3);
        assert_eq!(facts.placements().len(), 3);
        assert_eq!(facts.distinct_pages_and_extents(), 3);
    }

    fn root_and_block(
        record_count: u64,
    ) -> (PhysicalRootSourceCandidate, PhysicalManifestBlockProjection) {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let placement = placement(1);
        let block = PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![placement], 4).unwrap();
        let bytes = block.encode(format);
        let reference = block.reference(durable_artifact_checksum(&bytes));
        let free_key = FreeSpaceKey::new(RecordAllocationClass::Extent, 1).unwrap();
        let free = FreeSpaceBlockReference::new(1, 1, 0, 17, free_key, free_key).unwrap();
        let manifest = DurablePhysicalRootManifest::builder(1, 7, 4, 19)
            .record_count(record_count)
            .next_block(2)
            .routing_root(Some(reference))
            .free_space_root(Some(free))
            .admit()
            .unwrap();
        let selector = DurableRootSelector::new(
            store(),
            format,
            RootSelectorIdentity::new(1).unwrap(),
            RootSelectorRole::Current,
            1,
            None,
            None,
        )
        .unwrap();
        let observation = observe_structured_physical_root_candidate(selector, manifest, format);
        let PhysicalRootSlotObservation::Candidate(root) = observation else {
            panic!("root fixture must be admitted")
        };
        (
            root,
            PhysicalManifestBlockProjection::from_projected_block(reference, block),
        )
    }

    fn branched_root_and_blocks() -> (
        PhysicalRootSourceCandidate,
        Vec<PhysicalManifestBlockProjection>,
    ) {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let left = PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![placement(1)], 2).unwrap();
        let right =
            PhysicalRootRoutingBlock::leaf(7, 1, 2, vec![placement(2), placement(3)], 2).unwrap();
        let left_bytes = left.encode(format);
        let right_bytes = right.encode(format);
        let left_reference = left.reference(durable_artifact_checksum(&left_bytes));
        let right_reference = right.reference(durable_artifact_checksum(&right_bytes));
        let branch =
            PhysicalRootRoutingBlock::branch(7, 1, 3, 1, vec![left_reference, right_reference], 2)
                .unwrap();
        let branch_bytes = branch.encode(format);
        let branch_reference = branch.reference(durable_artifact_checksum(&branch_bytes));
        let free_key = FreeSpaceKey::new(RecordAllocationClass::Extent, 1).unwrap();
        let free = FreeSpaceBlockReference::new(1, 1, 0, 17, free_key, free_key).unwrap();
        let manifest = DurablePhysicalRootManifest::builder(1, 7, 2, 19)
            .record_count(3)
            .next_block(4)
            .routing_root(Some(branch_reference))
            .free_space_root(Some(free))
            .admit()
            .unwrap();
        let selector = DurableRootSelector::new(
            store(),
            format,
            RootSelectorIdentity::new(1).unwrap(),
            RootSelectorRole::Current,
            1,
            None,
            None,
        )
        .unwrap();
        let observation = observe_structured_physical_root_candidate(selector, manifest, format);
        let PhysicalRootSlotObservation::Candidate(root) = observation else {
            panic!("branched root fixture must be admitted")
        };
        (
            root,
            vec![
                PhysicalManifestBlockProjection::from_projected_block(branch_reference, branch),
                PhysicalManifestBlockProjection::from_projected_block(left_reference, left),
                PhysicalManifestBlockProjection::from_projected_block(right_reference, right),
            ],
        )
    }

    fn placement(ordinal: u64) -> CurrentPhysicalRecordPlacement {
        let record = PersistedRecordIdentity::new([9; 16], ordinal).unwrap();
        let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
            .record_extent_cell(PhysicalExtentId::from_raw(ordinal).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
        CurrentPhysicalRecordPlacement::Extent(
            DurableExtentRecordPlacement::new(record, extent, 23).unwrap(),
        )
    }

    fn store() -> StableStoreIdentity {
        StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap(),
        )
        .published_identity()
    }
}
