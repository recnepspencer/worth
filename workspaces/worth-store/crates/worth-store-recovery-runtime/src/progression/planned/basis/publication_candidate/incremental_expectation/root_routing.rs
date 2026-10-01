use worth_store_physical_format::{
    durable_artifact_checksum, CurrentPhysicalRecordPlacement, ManifestBlockReference,
    PersistedRecordIdentity, PhysicalRootRoutingBlock, RecordArtifactFile,
};

use super::super::{encoding, inventory, CandidateBuildDenial};
use super::CanonicalCandidateMatch;
use crate::progression::planned::basis::RecoveryBaseImagePlan;
use crate::progression::planned::PlanningResidentAllowance;

mod capacity_transition;

type Update = (PersistedRecordIdentity, CurrentPhysicalRecordPlacement);
type IndexedBlock<'a> = ((u64, u64), &'a PhysicalRootRoutingBlock);

pub(super) fn derive(
    matcher: &mut CanonicalCandidateMatch<'_>,
    base: &RecoveryBaseImagePlan,
    final_inventory: &inventory::FinalInventory,
    allowance: &mut PlanningResidentAllowance,
) -> Result<(Option<ManifestBlockReference>, u64), CandidateBuildDenial> {
    let selected = base.selected_root();
    if final_inventory.capacity != selected.node_capacity() {
        return capacity_transition::derive(matcher, base, final_inventory, allowance);
    }
    let (topology, updates) = indexed_source_and_updates(base, final_inventory, allowance)?;
    let mut planner = Planner {
        matcher,
        allowance,
        topology: &topology,
        generation: base.destination_generation(),
        capacity: final_inventory.capacity,
        tree: selected.tree_identity(),
        next_block: selected.next_block(),
    };
    let mut roots = match selected.routing_root() {
        Some(root) => planner.rewrite(root, &updates)?,
        None => {
            let mut entries = planner
                .allowance
                .reserve::<CurrentPhysicalRecordPlacement>(updates.len())?;
            entries.extend(updates.iter().map(|(_, placement)| *placement));
            let roots = planner.write_leaves(&entries)?;
            let bytes = PlanningResidentAllowance::vector_bytes(&entries)?;
            drop(entries);
            planner.allowance.release(bytes);
            roots
        }
    };
    while roots.len() > 1 {
        let next = planner.write_branches(&roots)?;
        let bytes = PlanningResidentAllowance::vector_bytes(&roots)?;
        drop(roots);
        planner.allowance.release(bytes);
        roots = next;
    }
    let root = roots.pop();
    let root_bytes = PlanningResidentAllowance::vector_bytes(&roots)?;
    drop(roots);
    planner.allowance.release(root_bytes);
    let next_block = planner.next_block;
    let topology_bytes = PlanningResidentAllowance::vector_bytes(&topology)?;
    let updates_bytes = PlanningResidentAllowance::vector_bytes(&updates)?;
    drop(planner);
    drop((topology, updates));
    allowance.release(topology_bytes);
    allowance.release(updates_bytes);
    Ok((root, next_block))
}

fn indexed_source_and_updates<'a>(
    base: &'a RecoveryBaseImagePlan,
    final_inventory: &inventory::FinalInventory,
    allowance: &mut PlanningResidentAllowance,
) -> Result<(Vec<IndexedBlock<'a>>, Vec<Update>), CandidateBuildDenial> {
    let mut topology =
        allowance.reserve::<IndexedBlock<'a>>(base.selected_root_topology().len())?;
    let selected_count = base
        .selected_root_topology()
        .iter()
        .filter_map(|(_, block)| block.entries())
        .try_fold(0usize, |count, entries| count.checked_add(entries.len()))
        .ok_or(CandidateBuildDenial::Invalid)?;
    let mut selected = allowance.reserve::<Update>(selected_count)?;
    for (reference, block) in base.selected_root_topology() {
        topology.push(((reference.generation(), reference.block()), block));
        if let Some(entries) = block.entries() {
            selected.extend(entries.iter().map(|entry| (entry.record(), *entry)));
        }
    }
    topology.sort_unstable_by_key(|(key, _)| *key);
    selected.sort_unstable_by_key(|(key, _)| *key);
    if topology.windows(2).any(|pair| pair[0].0 == pair[1].0)
        || selected.windows(2).any(|pair| pair[0].0 == pair[1].0)
    {
        return Err(CandidateBuildDenial::Invalid);
    }
    let mut updates = allowance.reserve::<Update>(final_inventory.placements.len())?;
    for entry in &final_inventory.placements {
        let key = entry.record();
        if selected
            .binary_search_by_key(&key, |(record, _)| *record)
            .ok()
            .is_none_or(|index| selected[index].1 != *entry)
        {
            updates.push((key, *entry));
        }
    }
    let bytes = PlanningResidentAllowance::vector_bytes(&selected)?;
    drop(selected);
    allowance.release(bytes);
    Ok((topology, updates))
}

struct Planner<'a, 'observed, 'topology> {
    matcher: &'a mut CanonicalCandidateMatch<'observed>,
    allowance: &'a mut PlanningResidentAllowance,
    topology: &'topology [IndexedBlock<'topology>],
    generation: u64,
    capacity: u16,
    tree: u64,
    next_block: u64,
}

impl Planner<'_, '_, '_> {
    fn rewrite(
        &mut self,
        reference: ManifestBlockReference,
        updates: &[Update],
    ) -> Result<Vec<ManifestBlockReference>, CandidateBuildDenial> {
        let key = (reference.generation(), reference.block());
        let index = self
            .topology
            .binary_search_by_key(&key, |(key, _)| *key)
            .map_err(|_| CandidateBuildDenial::Invalid)?;
        let block = self.topology[index].1;
        if let Some(entries) = block.entries() {
            let count = entries
                .len()
                .checked_add(updates.len())
                .ok_or(CandidateBuildDenial::Invalid)?;
            let mut merged = self
                .allowance
                .reserve::<CurrentPhysicalRecordPlacement>(count)?;
            merge_entries(entries, updates, &mut merged);
            let roots = self.write_leaves(&merged)?;
            let bytes = PlanningResidentAllowance::vector_bytes(&merged)?;
            drop(merged);
            self.allowance.release(bytes);
            return Ok(roots);
        }
        let children = block.children().ok_or(CandidateBuildDenial::Invalid)?;
        let mut rewritten = self
            .allowance
            .reserve::<ManifestBlockReference>(children.len())?;
        let mut start = 0;
        for (index, child) in children.iter().enumerate() {
            let end = if index + 1 == children.len() {
                updates.len()
            } else {
                start + updates[start..].partition_point(|(key, _)| *key <= child.last())
            };
            let child_updates = &updates[start..end];
            if child_updates.is_empty() {
                self.allowance.grow(&mut rewritten, 1)?;
                rewritten.push(*child);
            } else {
                let child_roots = self.rewrite(*child, child_updates)?;
                self.allowance.grow(&mut rewritten, child_roots.len())?;
                rewritten.extend_from_slice(&child_roots);
                let bytes = PlanningResidentAllowance::vector_bytes(&child_roots)?;
                drop(child_roots);
                self.allowance.release(bytes);
            }
            start = end;
        }
        let roots = self.write_branches(&rewritten)?;
        let bytes = PlanningResidentAllowance::vector_bytes(&rewritten)?;
        drop(rewritten);
        self.allowance.release(bytes);
        Ok(roots)
    }

    fn write_leaves(
        &mut self,
        entries: &[CurrentPhysicalRecordPlacement],
    ) -> Result<Vec<ManifestBlockReference>, CandidateBuildDenial> {
        let chunks = entries.chunks(usize::from(self.capacity));
        let mut roots = self
            .allowance
            .reserve::<ManifestBlockReference>(chunks.len())?;
        for chunk in chunks {
            let block_id = self.allocate()?;
            let mut copied = self
                .allowance
                .reserve::<CurrentPhysicalRecordPlacement>(chunk.len())?;
            copied.extend_from_slice(chunk);
            let block = encoding::root_leaf(
                self.tree,
                self.generation,
                block_id,
                copied,
                self.capacity,
                self.allowance,
            )?;
            let bytes = encoding::root_block(&block, self.matcher.format, self.allowance)?;
            roots.push(block.reference(durable_artifact_checksum(&bytes)));
            let block_bytes = block
                .owned_heap_bytes()
                .ok_or(CandidateBuildDenial::Invalid)?;
            drop(block);
            self.allowance.release(block_bytes);
            self.matcher.match_artifact(
                RecordArtifactFile::RootRoutingBlock {
                    generation: self.generation,
                    block: block_id,
                },
                bytes,
                self.allowance,
            )?;
        }
        Ok(roots)
    }

    fn write_branches(
        &mut self,
        children: &[ManifestBlockReference],
    ) -> Result<Vec<ManifestBlockReference>, CandidateBuildDenial> {
        let chunks = children.chunks(usize::from(self.capacity));
        let mut roots = self
            .allowance
            .reserve::<ManifestBlockReference>(chunks.len())?;
        for chunk in chunks {
            let block_id = self.allocate()?;
            let level = chunk[0]
                .level()
                .checked_add(1)
                .ok_or(CandidateBuildDenial::Invalid)?;
            let mut copied = self
                .allowance
                .reserve::<ManifestBlockReference>(chunk.len())?;
            copied.extend_from_slice(chunk);
            let block = PhysicalRootRoutingBlock::branch(
                self.tree,
                self.generation,
                block_id,
                level,
                copied,
                self.capacity,
            )
            .ok_or(CandidateBuildDenial::Invalid)?;
            let bytes = encoding::root_block(&block, self.matcher.format, self.allowance)?;
            roots.push(block.reference(durable_artifact_checksum(&bytes)));
            let block_bytes = block
                .owned_heap_bytes()
                .ok_or(CandidateBuildDenial::Invalid)?;
            drop(block);
            self.allowance.release(block_bytes);
            self.matcher.match_artifact(
                RecordArtifactFile::RootRoutingBlock {
                    generation: self.generation,
                    block: block_id,
                },
                bytes,
                self.allowance,
            )?;
        }
        Ok(roots)
    }

    fn allocate(&mut self) -> Result<u64, CandidateBuildDenial> {
        let block = self.next_block;
        self.next_block = block.checked_add(1).ok_or(CandidateBuildDenial::Invalid)?;
        Ok(block)
    }
}

fn merge_entries(
    entries: &[CurrentPhysicalRecordPlacement],
    updates: &[Update],
    merged: &mut Vec<CurrentPhysicalRecordPlacement>,
) {
    let mut existing = 0;
    let mut update = 0;
    while existing < entries.len() || update < updates.len() {
        match (entries.get(existing), updates.get(update)) {
            (Some(old), Some((key, _))) if old.record() < *key => {
                merged.push(*old);
                existing += 1;
            }
            (Some(old), Some((key, new))) if old.record() == *key => {
                merged.push(*new);
                existing += 1;
                update += 1;
            }
            (_, Some((_, new))) => {
                merged.push(*new);
                update += 1;
            }
            (Some(old), None) => {
                merged.push(*old);
                existing += 1;
            }
            (None, None) => break,
        }
    }
}
