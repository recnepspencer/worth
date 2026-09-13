use std::collections::BTreeMap;

use worth_store_physical_format::{
    durable_artifact_checksum, DurableFreeSpaceManifestHeader, FreeSpaceBlockReference,
    FreeSpaceKey, PhysicalFreeSpaceMembershipBlock, RecordArtifactFile,
    RecordFreeSpaceManifestEntry,
};

use super::super::{inventory, CandidateBuildDenial};
use super::CanonicalCandidateMatch;
use crate::progression::planned::basis::{RecoveryBaseImagePlan, RecoverySelectedSourceInventory};

#[derive(Clone, Copy)]
enum Update {
    Available(RecordFreeSpaceManifestEntry),
    Exhausted,
}

pub(super) fn derive(
    matcher: &mut CanonicalCandidateMatch<'_>,
    base: &RecoveryBaseImagePlan,
    source: &RecoverySelectedSourceInventory,
    final_inventory: &inventory::FinalInventory,
) -> Result<DurableFreeSpaceManifestHeader, CandidateBuildDenial> {
    let current = &source.free_space;
    let selected = source
        .free_entries
        .iter()
        .map(|entry| (FreeSpaceKey::from(*entry), *entry))
        .collect::<BTreeMap<_, _>>();
    let final_entries = final_inventory
        .free
        .iter()
        .map(|entry| (FreeSpaceKey::from(*entry), *entry))
        .collect::<BTreeMap<_, _>>();
    let mut updates = BTreeMap::new();
    for (key, entry) in &final_entries {
        if selected.get(key) != Some(entry) {
            updates.insert(*key, Update::Available(*entry));
        }
    }
    for key in selected.keys() {
        if !final_entries.contains_key(key) {
            updates.insert(*key, Update::Exhausted);
        }
    }
    let generation = base.destination_generation();
    let mut planner = Planner {
        matcher,
        topology: &source.free_topology,
        generation,
        capacity: final_inventory.capacity,
        tree: current.tree_identity(),
        next_block: current.next_block(),
    };
    let mut roots = match current.root() {
        Some(root) if final_inventory.capacity != current.node_capacity() => {
            planner.rewrite_all(root, &updates)?
        }
        Some(root) => planner.rewrite(root, &updates)?,
        None => planner.write_leaves(final_entries.into_values().collect())?,
    };
    while roots.len() > 1 {
        roots = planner.write_branches(roots)?;
    }
    DurableFreeSpaceManifestHeader::new(
        generation,
        current.tree_identity(),
        final_inventory.capacity,
        current.segment_page_capacity(),
        final_inventory.free.len() as u64,
        final_inventory.next_segment,
        final_inventory.next_page,
        final_inventory.next_extent,
        planner.next_block,
        roots.pop(),
    )
    .ok_or(CandidateBuildDenial::Invalid)
}

struct Planner<'matcher, 'observed, 'topology> {
    matcher: &'matcher mut CanonicalCandidateMatch<'observed>,
    topology: &'topology BTreeMap<(u64, u64), PhysicalFreeSpaceMembershipBlock>,
    generation: u64,
    capacity: u16,
    tree: u64,
    next_block: u64,
}

impl Planner<'_, '_, '_> {
    fn rewrite(
        &mut self,
        reference: FreeSpaceBlockReference,
        updates: &BTreeMap<FreeSpaceKey, Update>,
    ) -> Result<Vec<FreeSpaceBlockReference>, CandidateBuildDenial> {
        self.rewrite_inner(reference, updates, false)
    }

    fn rewrite_all(
        &mut self,
        reference: FreeSpaceBlockReference,
        updates: &BTreeMap<FreeSpaceKey, Update>,
    ) -> Result<Vec<FreeSpaceBlockReference>, CandidateBuildDenial> {
        self.rewrite_inner(reference, updates, true)
    }

    fn rewrite_inner(
        &mut self,
        reference: FreeSpaceBlockReference,
        updates: &BTreeMap<FreeSpaceKey, Update>,
        rewrite_all: bool,
    ) -> Result<Vec<FreeSpaceBlockReference>, CandidateBuildDenial> {
        let block = self
            .topology
            .get(&(reference.generation(), reference.block()))
            .ok_or(CandidateBuildDenial::Invalid)?;
        if let Some(entries) = block.entries() {
            let mut merged = entries
                .iter()
                .map(|entry| (FreeSpaceKey::from(*entry), *entry))
                .collect::<BTreeMap<_, _>>();
            apply(&mut merged, updates);
            self.write_leaves(merged.into_values().collect())
        } else {
            let children = block.children().ok_or(CandidateBuildDenial::Invalid)?;
            let assigned = assign(children, updates);
            let mut rewritten = Vec::new();
            for (child, child_updates) in children.iter().copied().zip(assigned) {
                if !rewrite_all && child_updates.is_empty() {
                    rewritten.push(child);
                } else {
                    rewritten.extend(self.rewrite_inner(child, &child_updates, rewrite_all)?);
                }
            }
            self.write_branches(rewritten)
        }
    }

    fn write_leaves(
        &mut self,
        entries: Vec<RecordFreeSpaceManifestEntry>,
    ) -> Result<Vec<FreeSpaceBlockReference>, CandidateBuildDenial> {
        let mut roots = Vec::new();
        for chunk in entries.chunks(usize::from(self.capacity)) {
            let block_id = self.allocate()?;
            let block = PhysicalFreeSpaceMembershipBlock::leaf(
                self.tree,
                self.generation,
                block_id,
                chunk.to_vec(),
                self.capacity,
            )
            .ok_or(CandidateBuildDenial::Invalid)?;
            let bytes = block.encode(self.matcher.format);
            roots.push(block.reference(durable_artifact_checksum(&bytes)));
            self.matcher.match_artifact(
                RecordArtifactFile::FreeSpaceMembershipBlock {
                    generation: self.generation,
                    block: block_id,
                },
                bytes,
            )?;
        }
        Ok(roots)
    }

    fn write_branches(
        &mut self,
        children: Vec<FreeSpaceBlockReference>,
    ) -> Result<Vec<FreeSpaceBlockReference>, CandidateBuildDenial> {
        let mut roots = Vec::new();
        for chunk in children.chunks(usize::from(self.capacity)) {
            let block_id = self.allocate()?;
            let block = PhysicalFreeSpaceMembershipBlock::branch(
                self.tree,
                self.generation,
                block_id,
                chunk[0]
                    .level()
                    .checked_add(1)
                    .ok_or(CandidateBuildDenial::Invalid)?,
                chunk.to_vec(),
                self.capacity,
            )
            .ok_or(CandidateBuildDenial::Invalid)?;
            let bytes = block.encode(self.matcher.format);
            roots.push(block.reference(durable_artifact_checksum(&bytes)));
            self.matcher.match_artifact(
                RecordArtifactFile::FreeSpaceMembershipBlock {
                    generation: self.generation,
                    block: block_id,
                },
                bytes,
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

fn apply(
    entries: &mut BTreeMap<FreeSpaceKey, RecordFreeSpaceManifestEntry>,
    updates: &BTreeMap<FreeSpaceKey, Update>,
) {
    for (key, update) in updates {
        match update {
            Update::Available(entry) => {
                entries.insert(*key, *entry);
            }
            Update::Exhausted => {
                entries.remove(key);
            }
        }
    }
}

fn assign(
    children: &[FreeSpaceBlockReference],
    updates: &BTreeMap<FreeSpaceKey, Update>,
) -> Vec<BTreeMap<FreeSpaceKey, Update>> {
    let mut assigned = vec![BTreeMap::new(); children.len()];
    for (key, update) in updates {
        let index = children
            .partition_point(|child| child.last() < *key)
            .min(children.len().saturating_sub(1));
        assigned[index].insert(*key, *update);
    }
    assigned
}
