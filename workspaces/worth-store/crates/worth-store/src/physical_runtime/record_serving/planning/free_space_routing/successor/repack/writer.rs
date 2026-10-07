use worth_store_physical_format::{
    durable_artifact_checksum, FreeSpaceBlockReference, PhysicalFreeSpaceMembershipBlock,
    RecordArtifactFile, RecordFreeSpaceManifestEntry,
};

use super::{
    damaged, metadata_pressure, AdmittedPhysicalRecordFormat, LocalRepackBudget, RecordAppendError,
};

pub(super) struct PackedWriter {
    format: AdmittedPhysicalRecordFormat,
    tree_identity: u64,
    generation: u64,
    capacity: usize,
    pub(super) next_block: u64,
    page_bytes: u64,
    max_blocks: usize,
    pub(super) emitted: u64,
    pub(super) blocks: Vec<(RecordArtifactFile, Vec<u8>)>,
    pending_entries: Vec<RecordFreeSpaceManifestEntry>,
    pending_levels: Vec<Vec<FreeSpaceBlockReference>>,
}

impl PackedWriter {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        format: AdmittedPhysicalRecordFormat,
        tree_identity: u64,
        generation: u64,
        capacity: u16,
        next_block: u64,
        entry_count: u64,
        max_blocks: usize,
        page_bytes: u64,
        budget: &mut LocalRepackBudget,
    ) -> Result<Self, RecordAppendError> {
        let level =
            worth_store_physical_format::required_tree_level(entry_count, capacity).unwrap_or(0);
        let level_slots = usize::from(level)
            .checked_add(2)
            .ok_or_else(metadata_pressure)?;
        let mut blocks = Vec::new();
        budget.reserve_vec(&mut blocks, max_blocks)?;
        let mut pending_entries = Vec::new();
        budget.reserve_vec(&mut pending_entries, usize::from(capacity))?;
        let mut pending_levels = Vec::new();
        budget.reserve_vec(&mut pending_levels, level_slots)?;
        for _ in 0..level_slots {
            let mut references = Vec::new();
            budget.reserve_vec(&mut references, usize::from(capacity))?;
            pending_levels.push(references);
        }
        Ok(Self {
            format,
            tree_identity,
            generation,
            capacity: usize::from(capacity),
            next_block,
            page_bytes,
            max_blocks,
            emitted: 0,
            blocks,
            pending_entries,
            pending_levels,
        })
    }

    pub(super) fn push(
        &mut self,
        entry: RecordFreeSpaceManifestEntry,
        budget: &mut LocalRepackBudget,
    ) -> Result<(), RecordAppendError> {
        if self.pending_entries.len() == self.pending_entries.capacity() {
            budget.reserve_vec(&mut self.pending_entries, self.capacity)?;
        }
        self.pending_entries.push(entry);
        self.emitted = self.emitted.checked_add(1).ok_or_else(damaged)?;
        if self.pending_entries.len() == self.capacity {
            self.flush_leaf(budget)?;
        }
        Ok(())
    }

    fn flush_leaf(&mut self, budget: &mut LocalRepackBudget) -> Result<(), RecordAppendError> {
        if self.pending_entries.is_empty() {
            return Ok(());
        }
        let entries = std::mem::take(&mut self.pending_entries);
        let block = PhysicalFreeSpaceMembershipBlock::leaf(
            self.tree_identity,
            self.generation,
            self.allocate_block()?,
            entries,
            self.capacity as u16,
        )
        .ok_or_else(damaged)?;
        let reference = self.stage(block, budget)?;
        self.push_reference(0, reference, budget)
    }

    fn push_reference(
        &mut self,
        level: usize,
        reference: FreeSpaceBlockReference,
        budget: &mut LocalRepackBudget,
    ) -> Result<(), RecordAppendError> {
        let pending = self.pending_levels.get_mut(level).ok_or_else(damaged)?;
        if pending.len() == pending.capacity() {
            budget.reserve_vec(pending, self.capacity)?;
        }
        pending.push(reference);
        if pending.len() == self.capacity {
            self.flush_level(level, budget)?;
        }
        Ok(())
    }

    fn flush_level(
        &mut self,
        level: usize,
        budget: &mut LocalRepackBudget,
    ) -> Result<(), RecordAppendError> {
        let pending = self.pending_levels.get_mut(level).ok_or_else(damaged)?;
        if pending.is_empty() {
            return Ok(());
        }
        let children = std::mem::take(pending);
        let block = PhysicalFreeSpaceMembershipBlock::branch(
            self.tree_identity,
            self.generation,
            self.allocate_block()?,
            u16::try_from(level + 1).map_err(|_| damaged())?,
            children,
            self.capacity as u16,
        )
        .ok_or_else(damaged)?;
        let reference = self.stage(block, budget)?;
        self.push_reference(level + 1, reference, budget)
    }

    pub(super) fn finish(
        &mut self,
        budget: &mut LocalRepackBudget,
    ) -> Result<Option<FreeSpaceBlockReference>, RecordAppendError> {
        self.flush_leaf(budget)?;
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
                .ok_or_else(damaged)?;
            self.flush_level(level, budget)?;
        }
    }

    fn allocate_block(&mut self) -> Result<u64, RecordAppendError> {
        let block = self.next_block;
        self.next_block = block.checked_add(1).ok_or_else(damaged)?;
        Ok(block)
    }

    fn stage(
        &mut self,
        block: PhysicalFreeSpaceMembershipBlock,
        budget: &mut LocalRepackBudget,
    ) -> Result<FreeSpaceBlockReference, RecordAppendError> {
        let length = block.encoded_frame_bytes().ok_or_else(damaged)?;
        if self.blocks.len() == self.max_blocks
            || u64::try_from(length).map_err(|_| metadata_pressure())? > self.page_bytes
        {
            return Err(metadata_pressure());
        }
        let mut reserved = Vec::new();
        budget.reserve_vec(&mut reserved, length)?;
        let reserved_capacity = reserved.capacity();
        let bytes = block
            .encode_in_reserved(self.format.declaration(), reserved)
            .ok_or_else(damaged)?;
        budget.reconcile_transfer(reserved_capacity, bytes.capacity())?;
        let reference = block.reference(durable_artifact_checksum(&bytes));
        let owned_capacity = match &block {
            PhysicalFreeSpaceMembershipBlock::Leaf { entries, .. } => (entries.capacity(), true),
            PhysicalFreeSpaceMembershipBlock::Branch { children, .. } => {
                (children.capacity(), false)
            }
        };
        self.blocks.push((
            RecordArtifactFile::FreeSpaceMembershipBlock {
                generation: self.generation,
                block: block.block(),
            },
            bytes,
        ));
        drop(block);
        if owned_capacity.1 {
            budget.release_capacity::<RecordFreeSpaceManifestEntry>(owned_capacity.0);
        } else {
            budget.release_capacity::<FreeSpaceBlockReference>(owned_capacity.0);
        }
        Ok(reference)
    }
}
