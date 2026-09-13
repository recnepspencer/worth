use std::collections::BTreeMap;

use worth_store_physical_format::{
    durable_artifact_checksum, CurrentPhysicalRecordPlacement, ManifestBlockReference,
    PersistedRecordIdentity, PhysicalRootRoutingBlock, RecordArtifactFile,
};

use super::super::super::{inventory, CandidateBuildDenial};
use super::super::CanonicalCandidateMatch;
use crate::progression::planned::basis::RecoveryBaseImagePlan;

pub(super) fn derive(
    matcher: &mut CanonicalCandidateMatch<'_>,
    base: &RecoveryBaseImagePlan,
    final_inventory: &inventory::FinalInventory,
) -> Result<(Option<ManifestBlockReference>, u64), CandidateBuildDenial> {
    let selected = base.selected_root();
    let topology = base
        .selected_root_topology()
        .iter()
        .cloned()
        .map(|(reference, block)| ((reference.generation(), reference.block()), block))
        .collect::<BTreeMap<_, _>>();
    let selected_entries = topology
        .values()
        .filter_map(PhysicalRootRoutingBlock::entries)
        .flatten()
        .map(|entry| (entry.record(), *entry))
        .collect::<BTreeMap<_, _>>();
    let updates = final_inventory
        .placements
        .iter()
        .filter(|entry| selected_entries.get(&entry.record()) != Some(entry))
        .map(|entry| (entry.record(), *entry))
        .collect::<BTreeMap<_, _>>();
    let mut writer = StreamingTreeWriter::new(
        matcher,
        selected.tree_identity(),
        base.destination_generation(),
        final_inventory.capacity,
        selected.next_block(),
    );
    let mut traversal = Traversal {
        topology: &topology,
        updates,
        writer: &mut writer,
    };
    if let Some(root) = selected.routing_root() {
        traversal.walk(root)?;
    }
    for (_, placement) in std::mem::take(&mut traversal.updates) {
        traversal.writer.push_entry(placement)?;
    }
    let root = traversal.writer.finish()?;
    Ok((root, traversal.writer.next_block))
}

struct Traversal<'topology, 'writer, 'matcher, 'observed> {
    topology: &'topology BTreeMap<(u64, u64), PhysicalRootRoutingBlock>,
    updates: BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    writer: &'writer mut StreamingTreeWriter<'matcher, 'observed>,
}

impl Traversal<'_, '_, '_, '_> {
    fn walk(&mut self, reference: ManifestBlockReference) -> Result<(), CandidateBuildDenial> {
        let block = self
            .topology
            .get(&(reference.generation(), reference.block()))
            .ok_or(CandidateBuildDenial::Invalid)?;
        if let Some(entries) = block.entries() {
            for existing in entries {
                while self
                    .updates
                    .first_key_value()
                    .is_some_and(|(record, _)| *record < existing.record())
                {
                    let (_, placement) = self.updates.pop_first().expect("first update exists");
                    self.writer.push_entry(placement)?;
                }
                self.writer
                    .push_entry(self.updates.remove(&existing.record()).unwrap_or(*existing))?;
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
        tree: u64,
        generation: u64,
        capacity: u16,
        next_block: u64,
    ) -> Self {
        Self {
            matcher,
            tree,
            generation,
            capacity: usize::from(capacity),
            next_block,
            pending_entries: Vec::with_capacity(usize::from(capacity)),
            pending_levels: Vec::new(),
        }
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
        let entries =
            std::mem::replace(&mut self.pending_entries, Vec::with_capacity(self.capacity));
        let block = PhysicalRootRoutingBlock::leaf(
            self.tree,
            self.generation,
            self.allocate()?,
            entries,
            self.capacity as u16,
        )
        .ok_or(CandidateBuildDenial::Invalid)?;
        let reference = self.stage(block)?;
        self.push_reference(0, reference)
    }

    fn push_reference(
        &mut self,
        level: usize,
        reference: ManifestBlockReference,
    ) -> Result<(), CandidateBuildDenial> {
        if self.pending_levels.len() <= level {
            self.pending_levels.resize_with(level + 1, Vec::new);
        }
        self.pending_levels[level].push(reference);
        if self.pending_levels[level].len() == self.capacity {
            self.flush_level(level)?;
        }
        Ok(())
    }

    fn flush_level(&mut self, level: usize) -> Result<(), CandidateBuildDenial> {
        let children = std::mem::take(&mut self.pending_levels[level]);
        let block = PhysicalRootRoutingBlock::branch(
            self.tree,
            self.generation,
            self.allocate()?,
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

    fn allocate(&mut self) -> Result<u64, CandidateBuildDenial> {
        let block = self.next_block;
        self.next_block = block.checked_add(1).ok_or(CandidateBuildDenial::Invalid)?;
        Ok(block)
    }

    fn stage(
        &mut self,
        block: PhysicalRootRoutingBlock,
    ) -> Result<ManifestBlockReference, CandidateBuildDenial> {
        let bytes = block.encode(self.matcher.format);
        let reference = block.reference(durable_artifact_checksum(&bytes));
        self.matcher.match_artifact(
            RecordArtifactFile::RootRoutingBlock {
                generation: self.generation,
                block: block.block(),
            },
            bytes,
        )?;
        Ok(reference)
    }
}
