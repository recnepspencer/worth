use std::collections::{BTreeMap, BTreeSet};

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, ManifestBlockReference,
    PersistedRecordIdentity, PhysicalRootRoutingBlock, RecordArtifactFile,
};

use super::{
    assignment, leaf_updates, ManifestDiscoveryCounterSnapshot, ManifestLookupFailure,
    ManifestReader,
};

/// Copy-on-write routing-block rewrite for one successor root.
pub(super) struct UpdatePlanner<'reader> {
    pub(super) allocation: &'reader worth_store_buffer_pool::OperationAllocationGrant,
    pub(super) reader: &'reader ManifestReader<'reader>,
    pub(super) current: &'reader DurablePhysicalRootManifest,
    pub(super) successor_generation: u64,
    pub(super) successor_capacity: u16,
    pub(super) next_block: u64,
    pub(super) blocks: Vec<(RecordArtifactFile, Vec<u8>)>,
    pub(super) discovery: ManifestDiscoveryCounterSnapshot,
    pub(super) inserted: u64,
    pub(super) removed: u64,
}

impl UpdatePlanner<'_> {
    pub(super) fn rewrite(
        &mut self,
        reference: ManifestBlockReference,
        updates: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
        drops: &BTreeSet<PersistedRecordIdentity>,
    ) -> Result<Vec<ManifestBlockReference>, ManifestLookupFailure> {
        let block = self
            .reader
            .read_block(self.allocation, reference, &mut self.discovery)?;
        match block {
            PhysicalRootRoutingBlock::Leaf { entries, .. } => {
                let merged = leaf_updates::merge_leaf(entries, updates, drops);
                self.inserted += merged.inserted;
                self.removed += merged.removed;
                self.write_leaves(merged.entries)
            }
            PhysicalRootRoutingBlock::Branch { children, .. } => {
                self.rewrite_children(children, updates, drops)
            }
        }
    }

    fn rewrite_children(
        &mut self,
        children: Vec<ManifestBlockReference>,
        updates: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
        drops: &BTreeSet<PersistedRecordIdentity>,
    ) -> Result<Vec<ManifestBlockReference>, ManifestLookupFailure> {
        let assigned = assignment::assign_updates(&children, updates, drops, &mut self.discovery);
        let mut rewritten = Vec::new();
        for (child, (child_updates, child_drops)) in children.into_iter().zip(assigned) {
            if child_updates.is_empty() && child_drops.is_empty() {
                rewritten.push(child);
            } else {
                rewritten.extend(self.rewrite(child, &child_updates, &child_drops)?);
            }
        }
        self.write_branch_level(rewritten)
    }

    pub(super) fn write_leaves(
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

    pub(super) fn write_parent_level(
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
