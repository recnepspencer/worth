#[cfg(test)]
use crate::history::data::BranchId;
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::access::LineageAccess;
#[cfg(test)]
use crate::lineage::data::LineageEventRecord;
use crate::lineage::data::LineageNode;
#[cfg(test)]
use crate::visibility::cache_state::cached_historical_state_for_version;
#[cfg(test)]
use std::collections::BTreeSet;

impl LineageAccess<'_> {
    pub fn for_record(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
    ) -> Result<Option<LineageNode>, crate::execution::RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        let mirror = self.runtime.acquire_partition_edition();
        let Some(head) = mirror.demand_partition_head(execution, entity_id.partition_id)? else {
            return Ok(None);
        };
        let view = head.head_view();
        let Some(slot) = view.entity(entity_id) else {
            return Ok(None);
        };
        let Some(id) = slot.extra().lineage_id else {
            return Ok(None);
        };
        let expected = EntityId::new(
            entity_id.partition_id,
            entity_id.local_slot.0,
            slot.generation(),
        );
        let Some(node) = self.runtime.lineage.demand_node(execution, id)? else {
            return Err(missing_required_node(id));
        };
        checked_record_node(node, expected).map(Some)
    }

    /// Capture resident facts only; native capture carries every unloaded page.
    pub(crate) fn resident_nodes_snapshot(&self) -> Vec<LineageNode> {
        self.runtime.lineage.nodes_snapshot()
    }

    #[cfg(test)]
    pub(crate) fn branch_events_snapshot(&self, branch_id: &BranchId) -> Vec<LineageEventRecord> {
        self.runtime.lineage.branch_events_snapshot(branch_id)
    }

    #[cfg(test)]
    pub(crate) fn branch_nodes_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_id: &BranchId,
    ) -> Result<Vec<LineageNode>, crate::execution::RelationalExecutionDenialCause> {
        let Some(head) = self.runtime.history().branch_head(branch_id) else {
            return Ok(Vec::new());
        };
        let cache_hit =
            cached_historical_state_for_version(self.runtime, head.version_id).is_some();
        let read_view = self
            .runtime
            .read_truth()
            .read_version(execution, head.version_id)?;
        let mut seen = BTreeSet::<LineageId>::new();
        let mut nodes = Vec::new();
        for record in read_view.entities() {
            let Some(id) = record.lineage_id else {
                continue;
            };
            if seen.insert(id) {
                let Some(node) = self.runtime.lineage.demand_node(execution, id)? else {
                    return Err(missing_required_node(id));
                };
                nodes.push(checked_record_node(node, record.entity_id)?);
            }
        }
        self.runtime
            .performance_access()
            .count_lineage_graph_snapshot_visibility_cache(cache_hit);
        Ok(nodes)
    }

    pub(crate) fn visible_entity_ids_for_lineages_for_observation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        lineage_ids: &[LineageId],
        observation: &crate::mvcc::RelationalBranchObservation,
    ) -> Result<Vec<EntityId>, crate::execution::RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        let mut nodes = Vec::new();
        for id in lineage_ids {
            if let Some(node) = self.runtime.lineage.demand_node(execution, *id)? {
                nodes.push(node);
            }
        }
        let root = observation.selected_root();
        root.demand_regions_full(
            execution,
            &self.runtime.services.symbols,
            nodes
                .iter()
                .map(|node| node.node().entity_id().partition_id),
        )?;
        let read_truth = self.runtime.read_truth();
        let mut entity_ids = Vec::new();
        for node in nodes {
            let Some(record) = read_truth.authoritative_entity_record_for_id_from_exact_state(
                root.as_ref(),
                root.schema_authority().registry(),
                node.node().entity_id(),
            ) else {
                continue;
            };
            if record.lineage_id != Some(node.node().lineage_id()) {
                return Err(node.reject_binding().into());
            }
            entity_ids.push(record.entity_id);
        }
        entity_ids.sort_unstable();
        entity_ids.dedup();
        Ok(entity_ids)
    }
}

fn checked_record_node(
    node: crate::runtime::ResidentLineageNode,
    expected: EntityId,
) -> Result<LineageNode, crate::execution::RelationalExecutionDenialCause> {
    if node.node().entity_id() != expected {
        return Err(node.reject_binding().into());
    }
    Ok(node.node().clone())
}

fn missing_required_node(id: LineageId) -> crate::execution::RelationalExecutionDenialCause {
    crate::execution::RelationalExecutionDenialCause::StorageUnavailable(
        crate::storage::data::RelationalStorageUnavailable {
            subject: crate::storage::data::RelationalStorageSubject::Lineage {
                first: id,
                last: id,
            },
            cause: crate::storage::data::RelationalStorageUnavailableCause::Damaged,
        },
    )
}
