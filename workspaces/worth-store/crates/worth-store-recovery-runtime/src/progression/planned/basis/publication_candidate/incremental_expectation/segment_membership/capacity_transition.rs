use std::collections::BTreeMap;

use worth_store_physical_format::{
    durable_artifact_checksum, PhysicalSegmentMembershipBlock, RecordArtifactFile,
    RecordSegmentPageManifestEntry, SegmentManifestBlockReference, SegmentPageKey,
};

use super::super::super::{inventory, CandidateBuildDenial};
use super::super::CanonicalCandidateMatch;
use crate::progression::planned::basis::{RecoveryBaseImagePlan, RecoverySelectedSourceInventory};

pub(super) fn derive(
    matcher: &mut CanonicalCandidateMatch<'_>,
    base: &RecoveryBaseImagePlan,
    source: &RecoverySelectedSourceInventory,
    updates: &BTreeMap<SegmentPageKey, RecordSegmentPageManifestEntry>,
    final_inventory: &inventory::FinalInventory,
) -> Result<(Option<SegmentManifestBlockReference>, u64), CandidateBuildDenial> {
    let selected = base.selected_root();
    let mut planner = Planner {
        matcher,
        topology: &source.segment_topology,
        generation: base.destination_generation(),
        capacity: final_inventory.capacity,
        tree: selected.tree_identity(),
        next_block: selected.next_segment_block(),
    };
    let mut roots = match selected.segment_root() {
        Some(root) => planner.rewrite_all(root, updates)?,
        None => planner.write_leaves(final_inventory.segments.to_vec())?,
    };
    while roots.len() > 1 {
        roots = planner.write_branches(roots)?;
    }
    Ok((roots.pop(), planner.next_block))
}

struct Planner<'planner, 'observed> {
    matcher: &'planner mut CanonicalCandidateMatch<'observed>,
    topology: &'planner BTreeMap<(u64, u64), PhysicalSegmentMembershipBlock>,
    generation: u64,
    capacity: u16,
    tree: u64,
    next_block: u64,
}

impl Planner<'_, '_> {
    fn rewrite_all(
        &mut self,
        reference: SegmentManifestBlockReference,
        updates: &BTreeMap<SegmentPageKey, RecordSegmentPageManifestEntry>,
    ) -> Result<Vec<SegmentManifestBlockReference>, CandidateBuildDenial> {
        let block = self
            .topology
            .get(&(reference.generation(), reference.block()))
            .ok_or(CandidateBuildDenial::Invalid)?;
        if let Some(entries) = block.entries() {
            let mut merged = entries
                .iter()
                .map(|entry| (SegmentPageKey::from(*entry), *entry))
                .collect::<BTreeMap<_, _>>();
            merged.extend(updates.iter().map(|(key, entry)| (*key, *entry)));
            return self.write_leaves(merged.into_values().collect());
        }
        let children = block.children().ok_or(CandidateBuildDenial::Invalid)?;
        let assigned = assign(children, updates);
        let mut rewritten = Vec::new();
        for (child, child_updates) in children.iter().copied().zip(assigned) {
            rewritten.extend(self.rewrite_all(child, &child_updates)?);
        }
        self.write_branches(rewritten)
    }

    fn write_leaves(
        &mut self,
        entries: Vec<RecordSegmentPageManifestEntry>,
    ) -> Result<Vec<SegmentManifestBlockReference>, CandidateBuildDenial> {
        let mut roots = Vec::new();
        for chunk in entries.chunks(usize::from(self.capacity)) {
            let block_id = self.allocate()?;
            let block = PhysicalSegmentMembershipBlock::leaf(
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
                RecordArtifactFile::SegmentMembershipBlock {
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
        children: Vec<SegmentManifestBlockReference>,
    ) -> Result<Vec<SegmentManifestBlockReference>, CandidateBuildDenial> {
        let mut roots = Vec::new();
        for chunk in children.chunks(usize::from(self.capacity)) {
            let block_id = self.allocate()?;
            let block = PhysicalSegmentMembershipBlock::branch(
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
                RecordArtifactFile::SegmentMembershipBlock {
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

fn assign(
    children: &[SegmentManifestBlockReference],
    updates: &BTreeMap<SegmentPageKey, RecordSegmentPageManifestEntry>,
) -> Vec<BTreeMap<SegmentPageKey, RecordSegmentPageManifestEntry>> {
    let mut assigned = vec![BTreeMap::new(); children.len()];
    for (key, entry) in updates {
        let index = children
            .partition_point(|child| child.last() < *key)
            .min(children.len().saturating_sub(1));
        assigned[index].insert(*key, *entry);
    }
    assigned
}
