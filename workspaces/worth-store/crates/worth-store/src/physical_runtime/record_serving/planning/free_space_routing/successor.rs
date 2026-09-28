use std::collections::BTreeMap;

use worth_store_physical_format::{
    durable_artifact_checksum, DurableFreeSpaceManifestHeader, FreeSpaceBlockReference,
    FreeSpaceKey, PhysicalFreeSpaceMembershipBlock, RecordArtifactFile,
    RecordFreeSpaceManifestEntry,
};

use super::super::super::access::manifest_routing::{
    ManifestDiscoveryCounterSnapshot, ManifestLookupFailure,
};
use super::super::super::{AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy};
use super::reader::FreeSpaceReader;

pub(in crate::physical_runtime::record_serving) enum FreeSpaceUpdate {
    Available(RecordFreeSpaceManifestEntry),
    Exhausted,
}

pub(in crate::physical_runtime::record_serving) struct FreeSpacePublicationPlan {
    pub(in crate::physical_runtime::record_serving) header: DurableFreeSpaceManifestHeader,
    pub(in crate::physical_runtime::record_serving) blocks: Vec<(RecordArtifactFile, Vec<u8>)>,
    pub(in crate::physical_runtime::record_serving) discovery: ManifestDiscoveryCounterSnapshot,
}

pub(in crate::physical_runtime::record_serving) struct FreeSpaceSuccessorRequest {
    pub(in crate::physical_runtime::record_serving) generation: u64,
    pub(in crate::physical_runtime::record_serving) node_capacity: u16,
    pub(in crate::physical_runtime::record_serving) segment_page_capacity: u32,
    pub(in crate::physical_runtime::record_serving) next_segment: u64,
    pub(in crate::physical_runtime::record_serving) next_page: u64,
    pub(in crate::physical_runtime::record_serving) next_extent: u64,
    pub(in crate::physical_runtime::record_serving) next_arena: u64,
    pub(in crate::physical_runtime::record_serving) updates:
        BTreeMap<FreeSpaceKey, FreeSpaceUpdate>,
}

impl FreeSpaceSuccessorRequest {
    fn segment_page_capacity(&self) -> u32 {
        self.segment_page_capacity
    }
}

pub(in crate::physical_runtime::record_serving) fn plan_free_space_successor(
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    residency: super::super::super::residency::PhysicalResidencyWorkPort,
    format: AdmittedPhysicalRecordFormat,
    access: AdmittedRecordAccessPolicy,
    current: &DurableFreeSpaceManifestHeader,
    request: FreeSpaceSuccessorRequest,
) -> Result<FreeSpacePublicationPlan, ManifestLookupFailure> {
    let segment_page_capacity = request.segment_page_capacity();
    if !super::super::policy_units::manifest_capacity_can_branch(request.node_capacity)
        || !super::super::policy_units::manifest_capacity_can_branch(current.node_capacity())
    {
        return Err(ManifestLookupFailure::Damaged);
    }
    let reader = FreeSpaceReader::serving(residency, format, access, current);
    let mut planner = FreeSpacePlanner {
        allocation,
        reader: &reader,
        generation: request.generation,
        node_capacity: request.node_capacity,
        next_block: current.next_block(),
        blocks: Vec::new(),
        discovery: ManifestDiscoveryCounterSnapshot::default(),
        inserted: 0,
        removed: 0,
    };
    let mut roots = match current.root() {
        Some(root) if request.node_capacity != current.node_capacity() => {
            planner.rewrite_all(root, &request.updates)?
        }
        Some(root) => planner.rewrite(root, &request.updates)?,
        None => {
            let mut entries = BTreeMap::new();
            planner.apply_updates(&mut entries, &request.updates);
            planner.write_leaves(entries.into_values().collect())?
        }
    };
    while roots.len() > 1 {
        roots = planner.write_branches(roots)?;
    }
    let entry_count = current
        .entry_count()
        .checked_add(planner.inserted)
        .and_then(|count| count.checked_sub(planner.removed))
        .ok_or(ManifestLookupFailure::Damaged)?;
    let header = DurableFreeSpaceManifestHeader::new(
        request.generation,
        current.tree_identity(),
        request.node_capacity,
        segment_page_capacity,
        entry_count,
        request.next_segment,
        request.next_page,
        request.next_extent,
        request.next_arena,
        current.arena_capacity(),
        current.arena_alignment(),
        planner.next_block,
        roots.pop(),
    )
    .ok_or(ManifestLookupFailure::Damaged)?;
    Ok(FreeSpacePublicationPlan {
        header,
        blocks: planner.blocks,
        discovery: planner.discovery,
    })
}

struct FreeSpacePlanner<'reader> {
    allocation: &'reader worth_store_buffer_pool::OperationAllocationGrant,
    reader: &'reader FreeSpaceReader<'reader>,
    generation: u64,
    node_capacity: u16,
    next_block: u64,
    blocks: Vec<(RecordArtifactFile, Vec<u8>)>,
    discovery: ManifestDiscoveryCounterSnapshot,
    inserted: u64,
    removed: u64,
}

impl FreeSpacePlanner<'_> {
    fn rewrite_all(
        &mut self,
        reference: FreeSpaceBlockReference,
        updates: &BTreeMap<FreeSpaceKey, FreeSpaceUpdate>,
    ) -> Result<Vec<FreeSpaceBlockReference>, ManifestLookupFailure> {
        match self
            .reader
            .read_block(self.allocation, reference, &mut self.discovery)?
        {
            PhysicalFreeSpaceMembershipBlock::Leaf { entries, .. } => {
                let mut merged = entries
                    .into_iter()
                    .map(|entry| (FreeSpaceKey::from(entry), entry))
                    .collect::<BTreeMap<_, _>>();
                self.apply_updates(&mut merged, updates);
                self.write_leaves(merged.into_values().collect())
            }
            PhysicalFreeSpaceMembershipBlock::Branch { children, .. } => {
                let assigned = self.assign_updates(&children, updates);
                let mut rewritten = Vec::new();
                for (child, child_updates) in children.into_iter().zip(assigned) {
                    rewritten.extend(self.rewrite_all(child, &child_updates)?);
                }
                self.write_branches(rewritten)
            }
        }
    }

    fn rewrite(
        &mut self,
        reference: FreeSpaceBlockReference,
        updates: &BTreeMap<FreeSpaceKey, FreeSpaceUpdate>,
    ) -> Result<Vec<FreeSpaceBlockReference>, ManifestLookupFailure> {
        match self
            .reader
            .read_block(self.allocation, reference, &mut self.discovery)?
        {
            PhysicalFreeSpaceMembershipBlock::Leaf { entries, .. } => {
                let mut merged = entries
                    .into_iter()
                    .map(|entry| (FreeSpaceKey::from(entry), entry))
                    .collect::<BTreeMap<_, _>>();
                self.apply_updates(&mut merged, updates);
                self.write_leaves(merged.into_values().collect())
            }
            PhysicalFreeSpaceMembershipBlock::Branch { children, .. } => {
                let assigned = self.assign_updates(&children, updates);
                let mut rewritten = Vec::new();
                for (child, child_updates) in children.into_iter().zip(assigned) {
                    if child_updates.is_empty() {
                        rewritten.push(child);
                    } else {
                        rewritten.extend(self.rewrite(child, &child_updates)?);
                    }
                }
                self.write_branches(rewritten)
            }
        }
    }

    fn apply_updates(
        &mut self,
        entries: &mut BTreeMap<FreeSpaceKey, RecordFreeSpaceManifestEntry>,
        updates: &BTreeMap<FreeSpaceKey, FreeSpaceUpdate>,
    ) {
        for (key, update) in updates {
            match update {
                FreeSpaceUpdate::Available(entry) => {
                    if entries.insert(*key, *entry).is_none() {
                        self.inserted += 1;
                    }
                }
                FreeSpaceUpdate::Exhausted => {
                    if entries.remove(key).is_some() {
                        self.removed += 1;
                    }
                }
            }
        }
    }

    fn assign_updates(
        &mut self,
        children: &[FreeSpaceBlockReference],
        updates: &BTreeMap<FreeSpaceKey, FreeSpaceUpdate>,
    ) -> Vec<BTreeMap<FreeSpaceKey, FreeSpaceUpdate>> {
        let mut assigned = std::iter::repeat_with(BTreeMap::new)
            .take(children.len())
            .collect::<Vec<_>>();
        for (key, update) in updates {
            let (index, comparisons) =
                super::super::super::access::counted_search::partition_point(children, |child| {
                    child.last() < *key
                });
            self.discovery.observe_comparisons(comparisons);
            let index = index.min(children.len().saturating_sub(1));
            assigned[index].insert(
                *key,
                match update {
                    FreeSpaceUpdate::Available(entry) => FreeSpaceUpdate::Available(*entry),
                    FreeSpaceUpdate::Exhausted => FreeSpaceUpdate::Exhausted,
                },
            );
        }
        assigned
    }

    fn write_leaves(
        &mut self,
        entries: Vec<RecordFreeSpaceManifestEntry>,
    ) -> Result<Vec<FreeSpaceBlockReference>, ManifestLookupFailure> {
        if entries.is_empty() {
            return Ok(Vec::new());
        }
        entries
            .chunks(usize::from(self.node_capacity))
            .map(|chunk| {
                let block = PhysicalFreeSpaceMembershipBlock::leaf(
                    self.reader.header.tree_identity(),
                    self.generation,
                    self.allocate_block()?,
                    chunk.to_vec(),
                    self.node_capacity,
                )
                .ok_or(ManifestLookupFailure::Damaged)?;
                Ok(self.stage(block))
            })
            .collect()
    }

    fn write_branches(
        &mut self,
        children: Vec<FreeSpaceBlockReference>,
    ) -> Result<Vec<FreeSpaceBlockReference>, ManifestLookupFailure> {
        children
            .chunks(usize::from(self.node_capacity))
            .map(|chunk| {
                let block = PhysicalFreeSpaceMembershipBlock::branch(
                    self.reader.header.tree_identity(),
                    self.generation,
                    self.allocate_block()?,
                    chunk[0]
                        .level()
                        .checked_add(1)
                        .ok_or(ManifestLookupFailure::Damaged)?,
                    chunk.to_vec(),
                    self.node_capacity,
                )
                .ok_or(ManifestLookupFailure::Damaged)?;
                Ok(self.stage(block))
            })
            .collect()
    }

    fn allocate_block(&mut self) -> Result<u64, ManifestLookupFailure> {
        let block = self.next_block;
        self.next_block = block.checked_add(1).ok_or(ManifestLookupFailure::Damaged)?;
        Ok(block)
    }

    fn stage(&mut self, block: PhysicalFreeSpaceMembershipBlock) -> FreeSpaceBlockReference {
        let bytes = block.encode(self.reader.format.declaration());
        let reference = block.reference(durable_artifact_checksum(&bytes));
        self.blocks.push((
            RecordArtifactFile::FreeSpaceMembershipBlock {
                generation: self.generation,
                block: block.block(),
            },
            bytes,
        ));
        reference
    }
}
