use worth_store_physical_format::{
    durable_artifact_checksum, DurableFreeSpaceManifestHeader, FreeSpaceBlockReference,
    FreeSpaceKey, PhysicalFreeSpaceMembershipBlock, RecordArtifactFile,
    RecordFreeSpaceManifestEntry,
};

use super::super::{encoding, inventory, CandidateBuildDenial};
use super::CanonicalCandidateMatch;
use crate::progression::planned::basis::{RecoveryBaseImagePlan, RecoverySelectedSourceInventory};
use crate::progression::planned::PlanningResidentAllowance;

#[derive(Clone, Copy)]
enum Change {
    Available(RecordFreeSpaceManifestEntry),
    Exhausted,
}
type Update = (FreeSpaceKey, Change);

pub(super) fn derive(
    matcher: &mut CanonicalCandidateMatch<'_>,
    base: &RecoveryBaseImagePlan,
    source: &RecoverySelectedSourceInventory,
    final_inventory: &inventory::FinalInventory,
    allowance: &mut PlanningResidentAllowance,
) -> Result<DurableFreeSpaceManifestHeader, CandidateBuildDenial> {
    let current = &source.free_space;
    let mut selected =
        allowance.reserve::<RecordFreeSpaceManifestEntry>(source.free_entries.len())?;
    selected.extend_from_slice(&source.free_entries);
    selected.sort_unstable_by_key(|entry| FreeSpaceKey::from(*entry));
    let max_updates = source
        .free_entries
        .len()
        .checked_add(final_inventory.free.len())
        .ok_or(CandidateBuildDenial::Invalid)?;
    let mut updates = allowance.reserve::<Update>(max_updates)?;
    diff_entries(&selected, &final_inventory.free, &mut updates);
    let selected_bytes = PlanningResidentAllowance::vector_bytes(&selected)?;
    drop(selected);
    allowance.release(selected_bytes);
    let generation = base.destination_generation();
    let mut planner = Planner {
        matcher,
        allowance,
        topology: &source.free_topology,
        generation,
        capacity: final_inventory.capacity,
        tree: current.tree_identity(),
        next_block: current.next_block(),
    };
    let mut roots = match current.root() {
        Some(root) => planner.rewrite(
            root,
            &updates,
            final_inventory.capacity != current.node_capacity(),
        )?,
        None => planner.write_leaves(&final_inventory.free)?,
    };
    while roots.len() > 1 {
        let next = planner.write_branches(&roots)?;
        let bytes = PlanningResidentAllowance::vector_bytes(&roots)?;
        drop(roots);
        planner.allowance.release(bytes);
        roots = next;
    }
    let root = roots.pop();
    let roots_bytes = PlanningResidentAllowance::vector_bytes(&roots)?;
    drop(roots);
    planner.allowance.release(roots_bytes);
    let next_block = planner.next_block;
    let update_bytes = PlanningResidentAllowance::vector_bytes(&updates)?;
    drop((planner, updates));
    allowance.release(update_bytes);
    DurableFreeSpaceManifestHeader::new_with_tier_epoch(
        generation,
        current.tree_identity(),
        final_inventory.capacity,
        current.segment_page_capacity(),
        final_inventory.free.len() as u64,
        final_inventory.next_segment,
        final_inventory.next_page,
        final_inventory.next_extent,
        final_inventory.next_arena,
        current.tier_epoch_start(),
        current.arena_capacity(),
        current.arena_alignment(),
        next_block,
        root,
    )
    .ok_or(CandidateBuildDenial::Invalid)
}

fn diff_entries(
    selected: &[RecordFreeSpaceManifestEntry],
    final_entries: &[RecordFreeSpaceManifestEntry],
    updates: &mut Vec<Update>,
) {
    let (mut old, mut new) = (0, 0);
    while old < selected.len() || new < final_entries.len() {
        match (selected.get(old), final_entries.get(new)) {
            (Some(before), Some(after))
                if FreeSpaceKey::from(*before) < FreeSpaceKey::from(*after) =>
            {
                updates.push((FreeSpaceKey::from(*before), Change::Exhausted));
                old += 1;
            }
            (Some(before), Some(after))
                if FreeSpaceKey::from(*before) == FreeSpaceKey::from(*after) =>
            {
                if before != after {
                    updates.push((FreeSpaceKey::from(*after), Change::Available(*after)));
                }
                old += 1;
                new += 1;
            }
            (_, Some(after)) => {
                updates.push((FreeSpaceKey::from(*after), Change::Available(*after)));
                new += 1;
            }
            (Some(before), None) => {
                updates.push((FreeSpaceKey::from(*before), Change::Exhausted));
                old += 1;
            }
            (None, None) => break,
        }
    }
}

struct Planner<'a, 'observed> {
    matcher: &'a mut CanonicalCandidateMatch<'observed>,
    allowance: &'a mut PlanningResidentAllowance,
    topology: &'a std::collections::BTreeMap<(u64, u64), PhysicalFreeSpaceMembershipBlock>,
    generation: u64,
    capacity: u16,
    tree: u64,
    next_block: u64,
}

impl Planner<'_, '_> {
    fn rewrite(
        &mut self,
        reference: FreeSpaceBlockReference,
        updates: &[Update],
        rewrite_all: bool,
    ) -> Result<Vec<FreeSpaceBlockReference>, CandidateBuildDenial> {
        let block = self
            .topology
            .get(&(reference.generation(), reference.block()))
            .ok_or(CandidateBuildDenial::Invalid)?;
        if let Some(entries) = block.entries() {
            let count = entries
                .len()
                .checked_add(updates.len())
                .ok_or(CandidateBuildDenial::Invalid)?;
            let mut merged = self
                .allowance
                .reserve::<RecordFreeSpaceManifestEntry>(count)?;
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
            .reserve::<FreeSpaceBlockReference>(children.len())?;
        let mut start = 0;
        for (index, child) in children.iter().enumerate() {
            let end = if index + 1 == children.len() {
                updates.len()
            } else {
                start + updates[start..].partition_point(|(key, _)| *key <= child.last())
            };
            let child_updates = &updates[start..end];
            if !rewrite_all && child_updates.is_empty() {
                self.allowance.grow(&mut rewritten, 1)?;
                rewritten.push(*child);
            } else {
                let child_roots = self.rewrite(*child, child_updates, rewrite_all)?;
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
        entries: &[RecordFreeSpaceManifestEntry],
    ) -> Result<Vec<FreeSpaceBlockReference>, CandidateBuildDenial> {
        let chunks = entries.chunks(usize::from(self.capacity));
        let mut roots = self
            .allowance
            .reserve::<FreeSpaceBlockReference>(chunks.len())?;
        for chunk in chunks {
            let block_id = self.allocate()?;
            let mut copied = self
                .allowance
                .reserve::<RecordFreeSpaceManifestEntry>(chunk.len())?;
            copied.extend_from_slice(chunk);
            let block = PhysicalFreeSpaceMembershipBlock::leaf(
                self.tree,
                self.generation,
                block_id,
                copied,
                self.capacity,
            )
            .ok_or(CandidateBuildDenial::Invalid)?;
            let bytes = encoding::free_block(&block, self.matcher.format, self.allowance)?;
            roots.push(block.reference(durable_artifact_checksum(&bytes)));
            let block_bytes = block
                .owned_heap_bytes()
                .ok_or(CandidateBuildDenial::Invalid)?;
            drop(block);
            self.allowance.release(block_bytes);
            self.matcher.match_artifact(
                RecordArtifactFile::FreeSpaceMembershipBlock {
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
        children: &[FreeSpaceBlockReference],
    ) -> Result<Vec<FreeSpaceBlockReference>, CandidateBuildDenial> {
        let chunks = children.chunks(usize::from(self.capacity));
        let mut roots = self
            .allowance
            .reserve::<FreeSpaceBlockReference>(chunks.len())?;
        for chunk in chunks {
            let block_id = self.allocate()?;
            let level = chunk[0]
                .level()
                .checked_add(1)
                .ok_or(CandidateBuildDenial::Invalid)?;
            let mut copied = self
                .allowance
                .reserve::<FreeSpaceBlockReference>(chunk.len())?;
            copied.extend_from_slice(chunk);
            let block = PhysicalFreeSpaceMembershipBlock::branch(
                self.tree,
                self.generation,
                block_id,
                level,
                copied,
                self.capacity,
            )
            .ok_or(CandidateBuildDenial::Invalid)?;
            let bytes = encoding::free_block(&block, self.matcher.format, self.allowance)?;
            roots.push(block.reference(durable_artifact_checksum(&bytes)));
            let block_bytes = block
                .owned_heap_bytes()
                .ok_or(CandidateBuildDenial::Invalid)?;
            drop(block);
            self.allowance.release(block_bytes);
            self.matcher.match_artifact(
                RecordArtifactFile::FreeSpaceMembershipBlock {
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
    entries: &[RecordFreeSpaceManifestEntry],
    updates: &[Update],
    merged: &mut Vec<RecordFreeSpaceManifestEntry>,
) {
    let (mut old, mut new) = (0, 0);
    while old < entries.len() || new < updates.len() {
        match (entries.get(old), updates.get(new)) {
            (Some(before), Some((key, _))) if FreeSpaceKey::from(*before) < *key => {
                merged.push(*before);
                old += 1;
            }
            (Some(before), Some((key, change))) if FreeSpaceKey::from(*before) == *key => {
                if let Change::Available(after) = change {
                    merged.push(*after);
                }
                old += 1;
                new += 1;
            }
            (_, Some((_, change))) => {
                if let Change::Available(after) = change {
                    merged.push(*after);
                }
                new += 1;
            }
            (Some(before), None) => {
                merged.push(*before);
                old += 1;
            }
            (None, None) => break,
        }
    }
}
