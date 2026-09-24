use std::collections::BTreeMap;

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, ManifestBlockReference,
    PersistedRecordIdentity, PhysicalRootRoutingBlock, RecordArtifactFile,
    SegmentManifestBlockReference,
};

use super::{ManifestDiscoveryCounterSnapshot, ManifestLookupFailure, ManifestReader};

pub(in crate::physical_runtime::record_serving) struct ManifestPublicationPlan {
    pub(in crate::physical_runtime::record_serving) root: DurablePhysicalRootManifest,
    pub(in crate::physical_runtime::record_serving) blocks: Vec<(RecordArtifactFile, Vec<u8>)>,
    pub(in crate::physical_runtime::record_serving) discovery: ManifestDiscoveryCounterSnapshot,
}

pub(in crate::physical_runtime::record_serving) struct RootManifestUpdateRequest<'updates> {
    pub(in crate::physical_runtime::record_serving) successor_generation: u64,
    pub(in crate::physical_runtime::record_serving) successor_capacity: u16,
    pub(in crate::physical_runtime::record_serving) free_space_checksum: u32,
    pub(in crate::physical_runtime::record_serving) free_space_root:
        Option<worth_store_physical_format::FreeSpaceBlockReference>,
    pub(in crate::physical_runtime::record_serving) segment_root:
        Option<SegmentManifestBlockReference>,
    pub(in crate::physical_runtime::record_serving) next_segment_block: u64,
    pub(in crate::physical_runtime::record_serving) placements:
        &'updates BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    pub(in crate::physical_runtime::record_serving) last_inline_record:
        Option<PersistedRecordIdentity>,
    pub(in crate::physical_runtime::record_serving) last_inline_segment:
        Option<worth_store_physical_format::SegmentGenerationCell>,
}

pub(in crate::physical_runtime::record_serving) fn plan_manifest_updates(
    reader: &ManifestReader<'_>,
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    current: &DurablePhysicalRootManifest,
    request: RootManifestUpdateRequest<'_>,
) -> Result<ManifestPublicationPlan, ManifestLookupFailure> {
    ManifestUpdatePlanning {
        reader,
        allocation,
        current,
        request,
    }
    .plan()
}

struct ManifestUpdatePlanning<'context, 'media, 'updates> {
    reader: &'context ManifestReader<'media>,
    allocation: &'context worth_store_buffer_pool::OperationAllocationGrant,
    current: &'context DurablePhysicalRootManifest,
    request: RootManifestUpdateRequest<'updates>,
}

impl ManifestUpdatePlanning<'_, '_, '_> {
    fn plan(self) -> Result<ManifestPublicationPlan, ManifestLookupFailure> {
        self.require_branchable_capacities()?;
        if self.request.successor_capacity != self.current.node_capacity() {
            if let Some(current_root) = self.current.routing_root() {
                return self.rebuild_for_successor_capacity(current_root);
            }
        }
        if let Some(plan) = self.incremental_when_packed()? {
            return Ok(plan);
        }
        let Some(current_root) = self.current.routing_root() else {
            return Err(ManifestLookupFailure::Damaged);
        };
        self.rebuild_for_successor_capacity(current_root)
    }

    fn require_branchable_capacities(&self) -> Result<(), ManifestLookupFailure> {
        let can_build_successor =
            super::super::super::planning::policy_units::manifest_capacity_can_branch(
                self.request.successor_capacity,
            );
        let can_read_current =
            super::super::super::planning::policy_units::manifest_capacity_can_branch(
                self.current.node_capacity(),
            );
        if can_build_successor && can_read_current {
            Ok(())
        } else {
            Err(ManifestLookupFailure::Damaged)
        }
    }

    fn rebuild_for_successor_capacity(
        self,
        current_root: ManifestBlockReference,
    ) -> Result<ManifestPublicationPlan, ManifestLookupFailure> {
        let Self {
            reader,
            allocation,
            current,
            request,
        } = self;
        let RootManifestUpdateRequest {
            successor_generation,
            successor_capacity,
            free_space_checksum,
            free_space_root,
            segment_root,
            next_segment_block,
            placements: updates,
            last_inline_record,
            last_inline_segment,
        } = request;
        let rebuilt = super::capacity_rebuild::rebuild_capacity(
            reader,
            allocation,
            super::capacity_rebuild::CapacityRebuildRequest {
                current_root,
                tree_identity: current.tree_identity(),
                successor_generation,
                successor_capacity,
                next_block: current.next_block(),
                updates: updates
                    .iter()
                    .map(|(record, placement)| (*record, *placement))
                    .collect(),
            },
        )?;
        let record_count = current
            .record_count()
            .checked_add(rebuilt.inserted)
            .ok_or(ManifestLookupFailure::Damaged)?;
        let root = DurablePhysicalRootManifest::builder(
            successor_generation,
            current.tree_identity(),
            successor_capacity,
            free_space_checksum,
        )
        .record_count(record_count)
        .next_block(rebuilt.next_block)
        .next_segment_block(next_segment_block)
        .routing_root(rebuilt.root)
        .segment_root(segment_root)
        .free_space_root(free_space_root)
        .last_inline_record(last_inline_record)
        .last_inline_segment(last_inline_segment)
        .admit()
        .ok_or(ManifestLookupFailure::Damaged)?;
        Ok(ManifestPublicationPlan {
            root,
            blocks: rebuilt.blocks,
            discovery: rebuilt.discovery,
        })
    }

    fn incremental_when_packed(
        &self,
    ) -> Result<Option<ManifestPublicationPlan>, ManifestLookupFailure> {
        let RootManifestUpdateRequest {
            successor_generation,
            successor_capacity,
            free_space_checksum,
            free_space_root,
            segment_root,
            next_segment_block,
            placements: updates,
            last_inline_record,
            last_inline_segment,
        } = self.request;
        let mut planner = UpdatePlanner {
            allocation: self.allocation,
            reader: self.reader,
            current: self.current,
            successor_generation,
            successor_capacity,
            next_block: self.current.next_block(),
            blocks: Vec::new(),
            discovery: ManifestDiscoveryCounterSnapshot::default(),
            inserted: 0,
        };
        let mut roots = match self.current.routing_root() {
            Some(root) => planner.rewrite(root, updates)?,
            None => {
                planner.inserted = updates.len() as u64;
                planner.write_leaves(updates.values().copied().collect())?
            }
        };
        while roots.len() > 1 {
            roots = planner.write_parent_level(roots)?;
        }
        let routing_root = roots.pop();
        let record_count = self
            .current
            .record_count()
            .checked_add(planner.inserted)
            .ok_or(ManifestLookupFailure::Damaged)?;
        let packed_level =
            worth_store_physical_format::required_tree_level(record_count, successor_capacity);
        if routing_root.map(ManifestBlockReference::level) != packed_level {
            return Ok(None);
        }
        let root = DurablePhysicalRootManifest::builder(
            successor_generation,
            self.current.tree_identity(),
            successor_capacity,
            free_space_checksum,
        )
        .record_count(record_count)
        .next_block(planner.next_block)
        .next_segment_block(next_segment_block)
        .routing_root(routing_root)
        .segment_root(segment_root)
        .free_space_root(free_space_root)
        .last_inline_record(last_inline_record)
        .last_inline_segment(last_inline_segment)
        .admit()
        .ok_or(ManifestLookupFailure::Damaged)?;
        Ok(Some(ManifestPublicationPlan {
            root,
            blocks: planner.blocks,
            discovery: planner.discovery,
        }))
    }
}

struct UpdatePlanner<'reader> {
    allocation: &'reader worth_store_buffer_pool::OperationAllocationGrant,
    reader: &'reader ManifestReader<'reader>,
    current: &'reader DurablePhysicalRootManifest,
    successor_generation: u64,
    successor_capacity: u16,
    next_block: u64,
    blocks: Vec<(RecordArtifactFile, Vec<u8>)>,
    discovery: ManifestDiscoveryCounterSnapshot,
    inserted: u64,
}

impl UpdatePlanner<'_> {
    fn rewrite(
        &mut self,
        reference: ManifestBlockReference,
        updates: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    ) -> Result<Vec<ManifestBlockReference>, ManifestLookupFailure> {
        let block = self
            .reader
            .read_block(self.allocation, reference, &mut self.discovery)?;
        match block {
            PhysicalRootRoutingBlock::Leaf { entries, .. } => {
                let mut merged = entries
                    .into_iter()
                    .map(|entry| (entry.record(), entry))
                    .collect::<BTreeMap<_, _>>();
                for (record, placement) in updates {
                    if merged.insert(*record, *placement).is_none() {
                        self.inserted += 1;
                    }
                }
                self.write_leaves(merged.into_values().collect())
            }
            PhysicalRootRoutingBlock::Branch { children, .. } => {
                self.rewrite_children(children, updates)
            }
        }
    }

    fn rewrite_children(
        &mut self,
        children: Vec<ManifestBlockReference>,
        updates: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    ) -> Result<Vec<ManifestBlockReference>, ManifestLookupFailure> {
        let assigned = self.assign_updates(&children, updates);
        let mut rewritten = Vec::new();
        for (child, child_updates) in children.into_iter().zip(assigned) {
            if child_updates.is_empty() {
                rewritten.push(child);
            } else {
                rewritten.extend(self.rewrite(child, &child_updates)?);
            }
        }
        self.write_branch_level(rewritten)
    }

    fn assign_updates(
        &mut self,
        children: &[ManifestBlockReference],
        updates: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    ) -> Vec<BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>> {
        let mut assigned = vec![BTreeMap::new(); children.len()];
        for (record, placement) in updates {
            let (index, comparisons) =
                super::super::counted_search::partition_point(children, |child| {
                    child.last() < *record
                });
            self.discovery.observe_comparisons(comparisons);
            let index = index.min(children.len().saturating_sub(1));
            assigned[index].insert(*record, *placement);
        }
        assigned
    }

    fn write_leaves(
        &mut self,
        entries: Vec<CurrentPhysicalRecordPlacement>,
    ) -> Result<Vec<ManifestBlockReference>, ManifestLookupFailure> {
        let capacity = usize::from(self.successor_capacity);
        entries
            .chunks(capacity)
            .map(|chunk| {
                let block_id = self.allocate_block()?;
                let block = PhysicalRootRoutingBlock::leaf(
                    self.current.tree_identity(),
                    self.successor_generation,
                    block_id,
                    chunk.to_vec(),
                    self.successor_capacity,
                )
                .ok_or(ManifestLookupFailure::Damaged)?;
                Ok(self.stage(block))
            })
            .collect()
    }

    fn write_branch_level(
        &mut self,
        children: Vec<ManifestBlockReference>,
    ) -> Result<Vec<ManifestBlockReference>, ManifestLookupFailure> {
        let capacity = usize::from(self.successor_capacity);
        children
            .chunks(capacity)
            .map(|chunk| {
                let block_id = self.allocate_block()?;
                let level = chunk[0]
                    .level()
                    .checked_add(1)
                    .ok_or(ManifestLookupFailure::Damaged)?;
                let block = PhysicalRootRoutingBlock::branch(
                    self.current.tree_identity(),
                    self.successor_generation,
                    block_id,
                    level,
                    chunk.to_vec(),
                    self.successor_capacity,
                )
                .ok_or(ManifestLookupFailure::Damaged)?;
                Ok(self.stage(block))
            })
            .collect()
    }

    fn write_parent_level(
        &mut self,
        children: Vec<ManifestBlockReference>,
    ) -> Result<Vec<ManifestBlockReference>, ManifestLookupFailure> {
        if children.len() == 1 {
            Ok(children)
        } else {
            self.write_branch_level(children)
        }
    }

    fn allocate_block(&mut self) -> Result<u64, ManifestLookupFailure> {
        let block = self.next_block;
        self.next_block = self
            .next_block
            .checked_add(1)
            .ok_or(ManifestLookupFailure::Damaged)?;
        Ok(block)
    }

    fn stage(&mut self, block: PhysicalRootRoutingBlock) -> ManifestBlockReference {
        let bytes = block.encode(self.reader.format_declaration());
        let reference = block.reference(worth_store_physical_format::durable_artifact_checksum(
            &bytes,
        ));
        self.blocks.push((
            RecordArtifactFile::RootRoutingBlock {
                generation: self.successor_generation,
                block: block.block(),
            },
            bytes,
        ));
        reference
    }
}
