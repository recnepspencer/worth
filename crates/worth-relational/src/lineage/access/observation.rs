//! Semantic node access on an exact admitted root, retaining the actual node page.
use crate::change_source::RelationalObservationReadDenial;
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::data::LineageNode;
use crate::mvcc::RelationalBranchObservation;
use crate::runtime::{RelationalRuntime, ResidentLineageNode};
use worth_execution::ExecutionRequest;

/// One immutable semantic node with its original counted page lease.
#[derive(Debug)]
pub struct RelationalLineageNodeRead {
    original: ResidentLineageNode,
}
impl RelationalLineageNodeRead {
    pub fn node(&self) -> &LineageNode {
        self.original.node()
    }
}

impl RelationalRuntime {
    /// Inspect one immutable lineage identity in this runtime's namespace.
    /// The counted read exposes semantic node DATA; it grants no branch
    /// membership, currentness, selected read root, or mutation authority.
    pub fn immutable_lineage_node(
        &self,
        request: &ExecutionRequest<'_, '_>,
        lineage: LineageId,
    ) -> Result<Option<RelationalLineageNodeRead>, crate::execution::RelationalExecutionDenialCause>
    {
        crate::execution::check_request(request)?;
        self.lineage
            .demand_node(request, lineage)
            .map(|node| node.map(|original| RelationalLineageNodeRead { original }))
            .map_err(crate::execution::RelationalExecutionDenialCause::from)
    }

    /// Read the node associated with an entity in this exact observation.
    /// Retired slots keep their lineage association; a different generation
    /// is absent. The read needs Head and the node page, without Full history.
    pub fn lineage_node_at_observation(
        &self,
        request: &ExecutionRequest<'_, '_>,
        observation: &RelationalBranchObservation,
        entity: EntityId,
    ) -> Result<Option<RelationalLineageNodeRead>, RelationalObservationReadDenial> {
        self.admit_observation_read(observation)?;
        self.lineage_node_for_admitted_observation(request, observation, entity)
            .map_err(RelationalObservationReadDenial::Execution)
    }

    pub(super) fn lineage_node_for_admitted_observation(
        &self,
        request: &ExecutionRequest<'_, '_>,
        observation: &RelationalBranchObservation,
        entity: EntityId,
    ) -> Result<Option<RelationalLineageNodeRead>, crate::execution::RelationalExecutionDenialCause>
    {
        crate::execution::check_request(request)?;
        let Some(head) = observation
            .selected_root()
            .demand_region_head(request, entity.partition_id)
            .map_err(crate::execution::RelationalExecutionDenialCause::from)?
        else {
            return Ok(None);
        };
        let view = head.head_view();
        let Some(slot) = view.entity(entity) else {
            return Ok(None);
        };
        if slot.generation() != entity.generation.0 {
            return Ok(None);
        }
        let Some(lineage) = slot.extra().lineage_id else {
            return Ok(None);
        };
        let read = self
            .immutable_lineage_node(request, lineage)?
            .ok_or_else(|| super::records::missing_required_node(lineage))?;
        if read.node().entity_id() != entity {
            return Err(read.original.reject_binding().into());
        }
        Ok(Some(read))
    }
}
