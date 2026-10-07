use worth_store_physical_format::{
    durable_artifact_checksum, CurrentPhysicalRecordPlacement, ManifestBlockReference,
    PhysicalRootRoutingBlock, RecordArtifactFile,
};

use super::super::super::{encoding, inventory, CandidateBuildDenial};
use super::super::CanonicalCandidateMatch;
use super::{indexed_source_and_updates, IndexedBlock, Update};
use crate::progression::planned::basis::RecoveryBaseImagePlan;
use crate::progression::planned::PlanningResidentAllowance;

pub(super) fn derive(
    matcher: &mut CanonicalCandidateMatch<'_>,
    base: &RecoveryBaseImagePlan,
    final_inventory: &inventory::FinalInventory,
    allowance: &mut PlanningResidentAllowance,
) -> Result<(Option<ManifestBlockReference>, u64), CandidateBuildDenial> {
    let selected = base.selected_root();
    let (topology, updates) = indexed_source_and_updates(base, final_inventory, allowance)?;
    let mut writer = StreamingTreeWriter::new(
        matcher,
        allowance,
        selected.tree_identity(),
        base.destination_generation(),
        final_inventory.capacity,
        selected.next_block(),
    )?;
    let mut traversal = Traversal {
        topology: &topology,
        updates: &updates,
        cursor: 0,
        writer: &mut writer,
    };
    if let Some(root) = selected.routing_root() {
        traversal.walk(root)?;
    }
    while let Some((_, placement)) = traversal.updates.get(traversal.cursor) {
        traversal.writer.push_entry(*placement)?;
        traversal.cursor += 1;
    }
    let root = traversal.writer.finish()?;
    let next_block = traversal.writer.next_block;
    drop(traversal);
    writer.release_storage()?;
    let topology_bytes = PlanningResidentAllowance::vector_bytes(&topology)?;
    let updates_bytes = PlanningResidentAllowance::vector_bytes(&updates)?;
    drop((topology, updates));
    allowance.release(topology_bytes);
    allowance.release(updates_bytes);
    Ok((root, next_block))
}

struct Traversal<'a, 'writer, 'matcher, 'observed> {
    topology: &'a [IndexedBlock<'a>],
    updates: &'a [Update],
    cursor: usize,
    writer: &'writer mut StreamingTreeWriter<'matcher, 'observed>,
}

impl Traversal<'_, '_, '_, '_> {
    fn walk(&mut self, reference: ManifestBlockReference) -> Result<(), CandidateBuildDenial> {
        let key = (reference.generation(), reference.block());
        let index = self
            .topology
            .binary_search_by_key(&key, |(key, _)| *key)
            .map_err(|_| CandidateBuildDenial::Invalid)?;
        let block = self.topology[index].1;
        if let Some(entries) = block.entries() {
            for existing in entries {
                while self
                    .updates
                    .get(self.cursor)
                    .is_some_and(|(key, _)| *key < existing.record())
                {
                    self.writer.push_entry(self.updates[self.cursor].1)?;
                    self.cursor += 1;
                }
                let placement = if self
                    .updates
                    .get(self.cursor)
                    .is_some_and(|(key, _)| *key == existing.record())
                {
                    let updated = self.updates[self.cursor].1;
                    self.cursor += 1;
                    updated
                } else {
                    *existing
                };
                self.writer.push_entry(placement)?;
            }
            return Ok(());
        }
        for child in block.children().ok_or(CandidateBuildDenial::Invalid)? {
            self.walk(*child)?;
        }
        Ok(())
    }
}

struct StreamingTreeWriter<'matcher, 'observed> {
    matcher: &'matcher mut CanonicalCandidateMatch<'observed>,
    allowance: &'matcher mut PlanningResidentAllowance,
    tree: u64,
    generation: u64,
    capacity: usize,
    next_block: u64,
    pending_entries: Vec<CurrentPhysicalRecordPlacement>,
    pending_levels: Vec<Vec<ManifestBlockReference>>,
}

impl<'matcher, 'observed> StreamingTreeWriter<'matcher, 'observed> {
    fn new(
        matcher: &'matcher mut CanonicalCandidateMatch<'observed>,
        allowance: &'matcher mut PlanningResidentAllowance,
        tree: u64,
        generation: u64,
        capacity: u16,
        next_block: u64,
    ) -> Result<Self, CandidateBuildDenial> {
        let pending_entries =
            allowance.reserve::<CurrentPhysicalRecordPlacement>(usize::from(capacity))?;
        Ok(Self {
            matcher,
            allowance,
            tree,
            generation,
            capacity: usize::from(capacity),
            next_block,
            pending_entries,
            pending_levels: Vec::new(),
        })
    }

    fn push_entry(
        &mut self,
        placement: CurrentPhysicalRecordPlacement,
    ) -> Result<(), CandidateBuildDenial> {
        self.pending_entries.push(placement);
        if self.pending_entries.len() == self.capacity {
            self.flush_leaf()?;
        }
        Ok(())
    }

    fn flush_leaf(&mut self) -> Result<(), CandidateBuildDenial> {
        if self.pending_entries.is_empty() {
            return Ok(());
        }
        let replacement = self
            .allowance
            .reserve::<CurrentPhysicalRecordPlacement>(self.capacity)?;
        let entries = std::mem::replace(&mut self.pending_entries, replacement);
        let block_id = self.allocate()?;
        let block = encoding::root_leaf(
            self.tree,
            self.generation,
            block_id,
            entries,
            self.capacity as u16,
            self.allowance,
        )?;
        let reference = self.stage(block)?;
        self.push_reference(0, reference)
    }

    fn push_reference(
        &mut self,
        level: usize,
        reference: ManifestBlockReference,
    ) -> Result<(), CandidateBuildDenial> {
        if self.pending_levels.len() <= level {
            let additional = level + 1 - self.pending_levels.len();
            self.allowance.grow(&mut self.pending_levels, additional)?;
            self.pending_levels.resize_with(level + 1, Vec::new);
        }
        self.allowance.grow(&mut self.pending_levels[level], 1)?;
        self.pending_levels[level].push(reference);
        if self.pending_levels[level].len() == self.capacity {
            self.flush_level(level)?;
        }
        Ok(())
    }

    fn flush_level(&mut self, level: usize) -> Result<(), CandidateBuildDenial> {
        let children = std::mem::take(&mut self.pending_levels[level]);
        let block_id = self.allocate()?;
        let block = PhysicalRootRoutingBlock::branch(
            self.tree,
            self.generation,
            block_id,
            u16::try_from(level + 1).map_err(|_| CandidateBuildDenial::Invalid)?,
            children,
            self.capacity as u16,
        )
        .ok_or(CandidateBuildDenial::Invalid)?;
        let reference = self.stage(block)?;
        self.push_reference(level + 1, reference)
    }

    fn finish(&mut self) -> Result<Option<ManifestBlockReference>, CandidateBuildDenial> {
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
                .ok_or(CandidateBuildDenial::Invalid)?;
            self.flush_level(level)?;
        }
    }

    fn stage(
        &mut self,
        block: PhysicalRootRoutingBlock,
    ) -> Result<ManifestBlockReference, CandidateBuildDenial> {
        let bytes = encoding::root_block(&block, self.matcher.format, self.allowance)?;
        let reference = block.reference(durable_artifact_checksum(&bytes));
        let artifact = RecordArtifactFile::RootRoutingBlock {
            generation: self.generation,
            block: block.block(),
        };
        let block_bytes = block
            .owned_heap_bytes()
            .ok_or(CandidateBuildDenial::Invalid)?;
        drop(block);
        self.allowance.release(block_bytes);
        self.matcher
            .match_artifact(artifact, bytes, self.allowance)?;
        Ok(reference)
    }

    fn allocate(&mut self) -> Result<u64, CandidateBuildDenial> {
        let block = self.next_block;
        self.next_block = block.checked_add(1).ok_or(CandidateBuildDenial::Invalid)?;
        Ok(block)
    }

    fn release_storage(self) -> Result<(), CandidateBuildDenial> {
        let entries_bytes = PlanningResidentAllowance::vector_bytes(&self.pending_entries)?;
        let levels_bytes = PlanningResidentAllowance::vector_bytes(&self.pending_levels)?;
        let child_bytes = self
            .pending_levels
            .iter()
            .try_fold(0u64, |total, level| {
                total.checked_add(PlanningResidentAllowance::vector_bytes(level).ok()?)
            })
            .ok_or(CandidateBuildDenial::Invalid)?;
        let Self {
            allowance,
            pending_entries,
            pending_levels,
            ..
        } = self;
        drop((pending_entries, pending_levels));
        allowance.release(entries_bytes);
        allowance.release(levels_bytes);
        allowance.release(child_bytes);
        Ok(())
    }
}
