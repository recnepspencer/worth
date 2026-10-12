#[cfg(test)]
use std::collections::BTreeSet;

use crate::capabilities::LineageNodeSource;
#[cfg(test)]
use crate::history::data::BranchId;
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::access::LineageAccess;
#[cfg(test)]
use crate::lineage::data::LineageEventRecord;
use crate::lineage::data::LineageNode;
#[cfg(test)]
use crate::visibility::cache_state::cached_historical_state_for_version;

impl<'runtime> LineageAccess<'runtime> {
    pub fn for_record(&self, entity_id: EntityId) -> Option<LineageNode> {
        let lineage_id = self
            .runtime
            .partitions
            .partition(entity_id.partition_id)?
            .entity_arena
            .get(&entity_id)
            .and_then(|slot_view| slot_view.extra().lineage_id)?;
        self.runtime.lineage_node(lineage_id)
    }

    pub(crate) fn nodes_snapshot(&self) -> Vec<LineageNode> {
        self.runtime.lineage_nodes_snapshot()
    }

    #[cfg(test)]
    pub(crate) fn branch_events_snapshot(
        &self,
        branch_id: &crate::history::data::BranchId,
    ) -> Vec<LineageEventRecord> {
        self.runtime.lineage.branch_events_snapshot(branch_id)
    }

    #[cfg(test)]
    pub(crate) fn branch_nodes_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_id: &BranchId,
    ) -> Vec<LineageNode> {
        let history = self.runtime.history();
        let Some(head) = history.branch_head(branch_id) else {
            return Vec::new();
        };
        let cache_hit =
            cached_historical_state_for_version(self.runtime, head.version_id).is_some();
        let read_view = self
            .runtime
            .read_truth()
            .read_version(execution, head.version_id);
        self.runtime
            .performance_access()
            .count_lineage_graph_snapshot_visibility_cache(cache_hit);
        let mut seen = BTreeSet::<LineageId>::new();
        let mut nodes = Vec::new();

        for record in read_view.entities() {
            let Some(lineage_id) = record.lineage_id else {
                continue;
            };
            let Some(node) = self.runtime.lineage_node(lineage_id) else {
                continue;
            };
            if seen.insert(node.lineage_id) {
                nodes.push(node);
            }
        }

        nodes
    }

    pub(crate) fn visible_entity_ids_for_lineages_for_observation(
        &self,
        lineage_ids: &[LineageId],
        observation: &crate::mvcc::RelationalBranchObservation,
    ) -> Vec<EntityId> {
        let read_truth = self.runtime.read_truth();
        let root = observation.selected_root();
        let mut entity_ids = lineage_ids
            .iter()
            .filter_map(|lineage_id| self.runtime.lineage_node(*lineage_id))
            .filter_map(|node| {
                read_truth
                    .authoritative_entity_record_for_id_from_exact_state(
                        root.as_ref(),
                        root.schema_authority().registry(),
                        node.entity_id(),
                    )
                    .map(|_| node.entity_id())
            })
            .collect::<Vec<_>>();
        entity_ids.sort_unstable_by_key(|entity_id| {
            (
                entity_id.partition_id.0,
                entity_id.local_slot.0,
                entity_id.generation.0,
            )
        });
        entity_ids.dedup();
        entity_ids
    }
}
