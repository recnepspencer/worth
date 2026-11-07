use std::collections::{BTreeMap, BTreeSet};

use crate::history::data::BranchId;
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::data::{LineageEventRecord, LineageNode};

#[derive(Debug, Clone, Default)]
pub(super) struct LineageResolutionIndexes {
    entity_lineages: BTreeMap<EntityId, LineageId>,
    branch_event_positions: BTreeMap<BranchId, Vec<usize>>,
    branch_source_event_positions: BTreeMap<BranchId, BTreeMap<LineageId, Vec<usize>>>,
    branch_lineage_event_positions: BTreeMap<BranchId, BTreeMap<LineageId, Vec<usize>>>,
    source_event_branches: BTreeMap<LineageId, BTreeSet<BranchId>>,
    lineage_event_branches: BTreeMap<LineageId, BTreeSet<BranchId>>,
}

impl LineageResolutionIndexes {
    pub(super) fn record_node(&mut self, node: &LineageNode) {
        self.entity_lineages
            .insert(node.entity_id(), node.lineage_id());
    }

    pub(super) fn append_event(&mut self, position: usize, event: &LineageEventRecord) {
        self.branch_event_positions
            .entry(event.branch_id().clone())
            .or_default()
            .push(position);
        let source_positions = self
            .branch_source_event_positions
            .entry(event.branch_id().clone())
            .or_default();
        for source in event.sources() {
            source_positions.entry(*source).or_default().push(position);
            self.source_event_branches
                .entry(*source)
                .or_default()
                .insert(event.branch_id().clone());
        }
        let lineage_positions = self
            .branch_lineage_event_positions
            .entry(event.branch_id().clone())
            .or_default();
        for lineage_id in event.sources().iter().chain(event.targets()) {
            lineage_positions
                .entry(*lineage_id)
                .or_default()
                .push(position);
            self.lineage_event_branches
                .entry(*lineage_id)
                .or_default()
                .insert(event.branch_id().clone());
        }
    }

    pub(super) fn rebuild<'a>(
        &mut self,
        _lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        nodes: impl IntoIterator<Item = &'a LineageNode>,
        events: impl IntoIterator<Item = &'a LineageEventRecord>,
    ) {
        *self = Self::default();
        for node in nodes {
            self.record_node(node);
        }
        for (position, event) in events.into_iter().enumerate() {
            self.append_event(position, event);
        }
    }

    #[cfg(test)]
    pub(super) fn branch_event_positions(&self, branch_id: &BranchId) -> &[usize] {
        self.branch_event_positions
            .get(branch_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub(super) fn branch_event_positions_for_sources(
        &self,
        branch_id: &BranchId,
        lineage_ids: &BTreeSet<LineageId>,
    ) -> BTreeSet<usize> {
        self.branch_source_event_positions
            .get(branch_id)
            .into_iter()
            .flat_map(|source_positions| {
                lineage_ids
                    .iter()
                    .filter_map(|lineage_id| source_positions.get(lineage_id))
            })
            .flat_map(|positions| positions.iter().copied())
            .collect()
    }

    pub(super) fn branch_event_positions_for_lineages(
        &self,
        branch_ids: &BTreeSet<BranchId>,
        lineage_ids: &BTreeSet<LineageId>,
        sources_only: bool,
    ) -> (BTreeSet<usize>, usize) {
        let mut positions = BTreeSet::new();
        let mut index_probe_count = 0;
        let branch_index = if sources_only {
            &self.branch_source_event_positions
        } else {
            &self.branch_lineage_event_positions
        };
        for branch_id in branch_ids {
            let Some(lineage_positions) = branch_index.get(branch_id) else {
                index_probe_count += lineage_ids.len();
                continue;
            };
            for lineage_id in lineage_ids {
                index_probe_count += 1;
                if let Some(event_positions) = lineage_positions.get(lineage_id) {
                    positions.extend(event_positions.iter().copied());
                }
            }
        }
        (positions, index_probe_count)
    }

    pub(super) fn lineage_for_entity(&self, entity_id: EntityId) -> (Option<LineageId>, usize) {
        (self.entity_lineages.get(&entity_id).copied(), 1)
    }

    pub(super) fn lineages_are_exclusive_to_branch(
        &self,
        lineage_ids: &BTreeSet<LineageId>,
        branch_id: &BranchId,
        sources_only: bool,
    ) -> (bool, usize) {
        let branch_index = if sources_only {
            &self.source_event_branches
        } else {
            &self.lineage_event_branches
        };
        (
            lineage_ids.iter().all(|lineage_id| {
                branch_index
                    .get(lineage_id)
                    .is_none_or(|branches| branches.len() == 1 && branches.contains(branch_id))
            }),
            lineage_ids.len(),
        )
    }
}
