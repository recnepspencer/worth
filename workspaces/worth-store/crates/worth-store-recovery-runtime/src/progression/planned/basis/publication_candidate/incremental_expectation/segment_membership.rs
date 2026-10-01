use worth_store_physical_format::{
    durable_artifact_checksum, PhysicalSegmentMembershipBlock, RecordArtifactFile,
    RecordSegmentPageManifestEntry, SegmentManifestBlockReference, SegmentPageKey,
};

use super::super::{encoding, inventory, CandidateBuildDenial};
use super::CanonicalCandidateMatch;
use crate::progression::planned::basis::{RecoveryBaseImagePlan, RecoverySelectedSourceInventory};
use crate::progression::planned::PlanningResidentAllowance;

mod capacity_transition;

type Update = (SegmentPageKey, RecordSegmentPageManifestEntry);

pub(super) fn derive(
    matcher: &mut CanonicalCandidateMatch<'_>,
    base: &RecoveryBaseImagePlan,
    source: &RecoverySelectedSourceInventory,
    final_inventory: &inventory::FinalInventory,
    allowance: &mut PlanningResidentAllowance,
) -> Result<(Option<SegmentManifestBlockReference>, u64), CandidateBuildDenial> {
    let mut selected = allowance.reserve::<Update>(source.segment_pages.len())?;
    selected.extend(
        source
            .segment_pages
            .values()
            .map(|page| (SegmentPageKey::from(page.entry), page.entry)),
    );
    selected.sort_unstable_by_key(|(key, _)| *key);
    let mut updates = allowance.reserve::<Update>(final_inventory.segments.len())?;
    for entry in &final_inventory.segments {
        let key = SegmentPageKey::from(*entry);
        if selected
            .binary_search_by_key(&key, |(key, _)| *key)
            .ok()
            .is_none_or(|index| selected[index].1 != *entry)
        {
            updates.push((key, *entry));
        }
    }
    let selected_bytes = PlanningResidentAllowance::vector_bytes(&selected)?;
    drop(selected);
    allowance.release(selected_bytes);
    let result = if final_inventory.capacity != base.selected_root().node_capacity() {
        capacity_transition::derive(matcher, base, source, final_inventory, &updates, allowance)
    } else if updates.is_empty() {
        Ok((
            base.selected_root().segment_root(),
            base.selected_root().next_segment_block(),
        ))
    } else {
        build_tree(
            matcher,
            base,
            source,
            final_inventory,
            &updates,
            false,
            allowance,
        )
    };
    let updates_bytes = PlanningResidentAllowance::vector_bytes(&updates)?;
    drop(updates);
    allowance.release(updates_bytes);
    result
}

fn build_tree(
    matcher: &mut CanonicalCandidateMatch<'_>,
    base: &RecoveryBaseImagePlan,
    source: &RecoverySelectedSourceInventory,
    final_inventory: &inventory::FinalInventory,
    updates: &[Update],
    rewrite_all: bool,
    allowance: &mut PlanningResidentAllowance,
) -> Result<(Option<SegmentManifestBlockReference>, u64), CandidateBuildDenial> {
    let selected = base.selected_root();
    let mut planner = Planner {
        matcher,
        allowance,
        topology: &source.segment_topology,
        generation: base.destination_generation(),
        capacity: final_inventory.capacity,
        tree: selected.tree_identity(),
        next_block: selected.next_segment_block(),
    };
    let mut roots = match selected.segment_root() {
        Some(root) => planner.rewrite(root, updates, rewrite_all)?,
        None => planner.write_leaves(&final_inventory.segments)?,
    };
    while roots.len() > 1 {
        let next = planner.write_branches(&roots)?;
        let bytes = PlanningResidentAllowance::vector_bytes(&roots)?;
        drop(roots);
        planner.allowance.release(bytes);
        roots = next;
    }
    let root = roots.pop();
    let bytes = PlanningResidentAllowance::vector_bytes(&roots)?;
    drop(roots);
    planner.allowance.release(bytes);
    Ok((root, planner.next_block))
}

struct Planner<'a, 'observed> {
    matcher: &'a mut CanonicalCandidateMatch<'observed>,
    allowance: &'a mut PlanningResidentAllowance,
    topology: &'a std::collections::BTreeMap<(u64, u64), PhysicalSegmentMembershipBlock>,
    generation: u64,
    capacity: u16,
    tree: u64,
    next_block: u64,
}

impl Planner<'_, '_> {
    fn rewrite(
        &mut self,
        reference: SegmentManifestBlockReference,
        updates: &[Update],
        rewrite_all: bool,
    ) -> Result<Vec<SegmentManifestBlockReference>, CandidateBuildDenial> {
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
                .reserve::<RecordSegmentPageManifestEntry>(count)?;
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
            .reserve::<SegmentManifestBlockReference>(children.len())?;
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
        entries: &[RecordSegmentPageManifestEntry],
    ) -> Result<Vec<SegmentManifestBlockReference>, CandidateBuildDenial> {
        let chunks = entries.chunks(usize::from(self.capacity));
        let mut roots = self
            .allowance
            .reserve::<SegmentManifestBlockReference>(chunks.len())?;
        for chunk in chunks {
            let block_id = self.allocate()?;
            let mut copied = self
                .allowance
                .reserve::<RecordSegmentPageManifestEntry>(chunk.len())?;
            copied.extend_from_slice(chunk);
            let block = PhysicalSegmentMembershipBlock::leaf(
                self.tree,
                self.generation,
                block_id,
                copied,
                self.capacity,
            )
            .ok_or(CandidateBuildDenial::Invalid)?;
            let bytes = encoding::segment_block(&block, self.matcher.format, self.allowance)?;
            roots.push(block.reference(durable_artifact_checksum(&bytes)));
            let block_bytes = block
                .owned_heap_bytes()
                .ok_or(CandidateBuildDenial::Invalid)?;
            drop(block);
            self.allowance.release(block_bytes);
            self.matcher.match_artifact(
                RecordArtifactFile::SegmentMembershipBlock {
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
        children: &[SegmentManifestBlockReference],
    ) -> Result<Vec<SegmentManifestBlockReference>, CandidateBuildDenial> {
        let chunks = children.chunks(usize::from(self.capacity));
        let mut roots = self
            .allowance
            .reserve::<SegmentManifestBlockReference>(chunks.len())?;
        for chunk in chunks {
            let block_id = self.allocate()?;
            let level = chunk[0]
                .level()
                .checked_add(1)
                .ok_or(CandidateBuildDenial::Invalid)?;
            let mut copied = self
                .allowance
                .reserve::<SegmentManifestBlockReference>(chunk.len())?;
            copied.extend_from_slice(chunk);
            let block = PhysicalSegmentMembershipBlock::branch(
                self.tree,
                self.generation,
                block_id,
                level,
                copied,
                self.capacity,
            )
            .ok_or(CandidateBuildDenial::Invalid)?;
            let bytes = encoding::segment_block(&block, self.matcher.format, self.allowance)?;
            roots.push(block.reference(durable_artifact_checksum(&bytes)));
            let block_bytes = block
                .owned_heap_bytes()
                .ok_or(CandidateBuildDenial::Invalid)?;
            drop(block);
            self.allowance.release(block_bytes);
            self.matcher.match_artifact(
                RecordArtifactFile::SegmentMembershipBlock {
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
    entries: &[RecordSegmentPageManifestEntry],
    updates: &[Update],
    merged: &mut Vec<RecordSegmentPageManifestEntry>,
) {
    let mut existing = 0;
    let mut update = 0;
    while existing < entries.len() || update < updates.len() {
        match (entries.get(existing), updates.get(update)) {
            (Some(old), Some((key, _))) if SegmentPageKey::from(*old) < *key => {
                merged.push(*old);
                existing += 1;
            }
            (Some(old), Some((key, new))) if SegmentPageKey::from(*old) == *key => {
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
