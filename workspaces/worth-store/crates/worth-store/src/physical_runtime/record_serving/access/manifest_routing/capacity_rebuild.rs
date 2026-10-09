use std::collections::{BTreeMap, BTreeSet};

use worth_store_physical_format::{
    durable_artifact_checksum, CurrentPhysicalRecordPlacement, ManifestBlockReference,
    PersistedRecordIdentity, PhysicalRootRoutingBlock, RecordArtifactFile,
};

use super::{ManifestDiscoveryCounterSnapshot, ManifestLookupFailure, ManifestReader};

pub(super) struct CapacityRebuild {
    pub(super) root: Option<ManifestBlockReference>,
    pub(super) next_block: u64,
    pub(super) blocks: Vec<(RecordArtifactFile, Vec<u8>)>,
    pub(super) discovery: ManifestDiscoveryCounterSnapshot,
    pub(super) inserted: u64,
    pub(super) removed: u64,
}

pub(super) struct CapacityRebuildRequest {
    pub(super) current_root: ManifestBlockReference,
    pub(super) tree_identity: u64,
    pub(super) successor_generation: u64,
    pub(super) successor_capacity: u16,
    pub(super) next_block: u64,
    pub(super) updates: BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    pub(super) drops: BTreeSet<PersistedRecordIdentity>,
}

pub(super) fn rebuild_capacity(
    reader: &ManifestReader<'_>,
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    request: CapacityRebuildRequest,
) -> Result<CapacityRebuild, ManifestLookupFailure> {
    let CapacityRebuildRequest {
        current_root,
        tree_identity,
        successor_generation,
        successor_capacity,
        next_block,
        updates,
        drops,
    } = request;
    if !super::super::super::planning::policy_units::manifest_capacity_can_branch(
        successor_capacity,
    ) {
        return Err(ManifestLookupFailure::Damaged);
    }
    let writer = StreamingTreeWriter::new(StreamingTreeDeclaration {
        format: reader.format_declaration(),
        tree_identity,
        generation: successor_generation,
        capacity: successor_capacity,
        next_block,
    });
    let mut traversal = CapacityRebuildTraversal {
        reader,
        allocation,
        discovery: ManifestDiscoveryCounterSnapshot::default(),
        updates,
        drops,
        inserted: 0,
        removed: 0,
        writer,
    };
    traversal.walk(current_root)?;
    traversal.finish()
}

struct CapacityRebuildTraversal<'context, 'media> {
    reader: &'context ManifestReader<'media>,
    allocation: &'context worth_store_buffer_pool::OperationAllocationGrant,
    discovery: ManifestDiscoveryCounterSnapshot,
    updates: BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    drops: BTreeSet<PersistedRecordIdentity>,
    inserted: u64,
    removed: u64,
    writer: StreamingTreeWriter,
}

impl CapacityRebuildTraversal<'_, '_> {
    fn walk(&mut self, reference: ManifestBlockReference) -> Result<(), ManifestLookupFailure> {
        match self
            .reader
            .read_block(self.allocation, reference, &mut self.discovery)?
        {
            PhysicalRootRoutingBlock::Leaf { entries, .. } => {
                for existing in entries {
                    while self
                        .updates
                        .first_key_value()
                        .is_some_and(|(record, _)| *record < existing.record())
                    {
                        let (_, placement) = self.updates.pop_first().expect("first update exists");
                        self.inserted = self.inserted.saturating_add(1);
                        self.writer.push_entry(placement)?;
                    }
                    if self.drops.remove(&existing.record()) {
                        self.removed = self
                            .removed
                            .checked_add(1)
                            .ok_or(ManifestLookupFailure::Damaged)?;
                    } else {
                        let selected = self.updates.remove(&existing.record()).unwrap_or(existing);
                        self.writer.push_entry(selected)?;
                    }
                }
                Ok(())
            }
            PhysicalRootRoutingBlock::Branch { children, .. } => {
                for child in children {
                    self.walk(child)?;
                }
                Ok(())
            }
        }
    }

    fn finish(mut self) -> Result<CapacityRebuild, ManifestLookupFailure> {
        if !self.drops.is_empty() {
            return Err(ManifestLookupFailure::Damaged);
        }
        for (_, placement) in std::mem::take(&mut self.updates) {
            self.inserted = self.inserted.saturating_add(1);
            self.writer.push_entry(placement)?;
        }
        let root = self.writer.finish()?;
        Ok(CapacityRebuild {
            root,
            next_block: self.writer.next_block,
            blocks: self.writer.blocks,
            discovery: self.discovery,
            inserted: self.inserted,
            removed: self.removed,
        })
    }
}

struct StreamingTreeDeclaration {
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    tree_identity: u64,
    generation: u64,
    capacity: u16,
    next_block: u64,
}

struct StreamingTreeWriter {
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    tree_identity: u64,
    generation: u64,
    capacity: usize,
    next_block: u64,
    blocks: Vec<(RecordArtifactFile, Vec<u8>)>,
    pending_entries: Vec<CurrentPhysicalRecordPlacement>,
    pending_levels: Vec<Vec<ManifestBlockReference>>,
}

impl StreamingTreeWriter {
    fn new(declaration: StreamingTreeDeclaration) -> Self {
        Self {
            format: declaration.format,
            tree_identity: declaration.tree_identity,
            generation: declaration.generation,
            capacity: usize::from(declaration.capacity),
            next_block: declaration.next_block,
            blocks: Vec::new(),
            pending_entries: Vec::with_capacity(usize::from(declaration.capacity)),
            pending_levels: Vec::new(),
        }
    }

    fn push_entry(
        &mut self,
        placement: CurrentPhysicalRecordPlacement,
    ) -> Result<(), ManifestLookupFailure> {
        self.pending_entries.push(placement);
        if self.pending_entries.len() == self.capacity {
            self.flush_leaf()?;
        }
        Ok(())
    }

    fn flush_leaf(&mut self) -> Result<(), ManifestLookupFailure> {
        if self.pending_entries.is_empty() {
            return Ok(());
        }
        let entries =
            std::mem::replace(&mut self.pending_entries, Vec::with_capacity(self.capacity));
        let block = PhysicalRootRoutingBlock::leaf(
            self.tree_identity,
            self.generation,
            self.allocate_block()?,
            entries,
            self.capacity as u16,
        )
        .ok_or(ManifestLookupFailure::Damaged)?;
        let reference = self.stage(block);
        self.push_reference(0, reference)
    }

    fn push_reference(
        &mut self,
        level: usize,
        reference: ManifestBlockReference,
    ) -> Result<(), ManifestLookupFailure> {
        if self.pending_levels.len() <= level {
            self.pending_levels.resize_with(level + 1, Vec::new);
        }
        self.pending_levels[level].push(reference);
        if self.pending_levels[level].len() == self.capacity {
            self.flush_level(level)?;
        }
        Ok(())
    }

    fn flush_level(&mut self, level: usize) -> Result<(), ManifestLookupFailure> {
        let children = std::mem::take(&mut self.pending_levels[level]);
        let block = PhysicalRootRoutingBlock::branch(
            self.tree_identity,
            self.generation,
            self.allocate_block()?,
            u16::try_from(level + 1).map_err(|_| ManifestLookupFailure::Damaged)?,
            children,
            self.capacity as u16,
        )
        .ok_or(ManifestLookupFailure::Damaged)?;
        let reference = self.stage(block);
        self.push_reference(level + 1, reference)
    }

    fn finish(&mut self) -> Result<Option<ManifestBlockReference>, ManifestLookupFailure> {
        self.flush_leaf()?;
        loop {
            let total = self.pending_levels.iter().map(Vec::len).sum::<usize>();
            if total == 0 {
                return Ok(None);
            }
            if total == 1 {
                return Ok(self.pending_levels.iter_mut().find_map(Vec::pop));
            }
            let level = self
                .pending_levels
                .iter()
                .position(|references| !references.is_empty())
                .expect("a pending level exists");
            self.flush_level(level)?;
        }
    }

    fn allocate_block(&mut self) -> Result<u64, ManifestLookupFailure> {
        let block = self.next_block;
        self.next_block = block.checked_add(1).ok_or(ManifestLookupFailure::Damaged)?;
        Ok(block)
    }

    fn stage(&mut self, block: PhysicalRootRoutingBlock) -> ManifestBlockReference {
        let bytes = block.encode(self.format);
        let reference = block.reference(durable_artifact_checksum(&bytes));
        self.blocks.push((
            RecordArtifactFile::RootRoutingBlock {
                generation: self.generation,
                block: block.block(),
            },
            bytes,
        ));
        reference
    }
}
