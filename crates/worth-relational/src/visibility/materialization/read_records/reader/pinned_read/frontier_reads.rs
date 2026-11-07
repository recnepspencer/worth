//! Frontier work uses one captured exact root and preserves caller interruption.
use super::super::*;
use super::*;
use crate::identity::data::{EntityId, KindId};
use crate::storage::partition::AdjacencyDirection;
use std::collections::BTreeSet;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
impl<Snapshot> RelationalPinnedRead<Snapshot> {
    pub fn bounded_outgoing_relations_for_frontier(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        entities: &BTreeSet<EntityId>,
        kind: KindId,
        maximum: Option<usize>,
    ) -> Result<
        BoundedFrontierAdjacencyTruthRead,
        BoundedFrontierReadStop<
            FrontierAdjacencyTruthReadLimitExceeded,
            crate::execution::RelationalExecutionDenialCause,
        >,
    > {
        self.frontier_relations(
            request,
            entities,
            kind,
            maximum,
            AdjacencyDirection::Outgoing,
            false,
        )
    }
    pub fn bounded_incoming_relations_for_frontier(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        entities: &BTreeSet<EntityId>,
        kind: KindId,
        maximum: Option<usize>,
    ) -> Result<
        BoundedFrontierAdjacencyTruthRead,
        BoundedFrontierReadStop<
            FrontierAdjacencyTruthReadLimitExceeded,
            crate::execution::RelationalExecutionDenialCause,
        >,
    > {
        self.frontier_relations(
            request,
            entities,
            kind,
            maximum,
            AdjacencyDirection::Incoming,
            false,
        )
    }
    /// Read live and audit-retained relation identities from this exact basis.
    /// Endpoint retirement does not erase historical correspondence. This does
    /// not revive an entity or confer permission to act on it.
    pub fn bounded_outgoing_relations_including_audit_for_frontier(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        entities: &BTreeSet<EntityId>,
        kind: KindId,
        maximum: Option<usize>,
    ) -> Result<
        BoundedFrontierAdjacencyTruthRead,
        BoundedFrontierReadStop<
            FrontierAdjacencyTruthReadLimitExceeded,
            crate::execution::RelationalExecutionDenialCause,
        >,
    > {
        self.frontier_relations(
            request,
            entities,
            kind,
            maximum,
            AdjacencyDirection::Outgoing,
            true,
        )
    }
    fn frontier_relations(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        entities: &BTreeSet<EntityId>,
        kind: KindId,
        maximum: Option<usize>,
        direction: AdjacencyDirection,
        include_audit: bool,
    ) -> Result<
        BoundedFrontierAdjacencyTruthRead,
        BoundedFrontierReadStop<
            FrontierAdjacencyTruthReadLimitExceeded,
            crate::execution::RelationalExecutionDenialCause,
        >,
    > {
        self.project_read()
            .bounded_relations_for_frontier(
                request,
                entities,
                kind,
                direction,
                maximum,
                include_audit,
            )
            .map_err(BoundedFrontierReadStop::Admission)?
            .map_err(BoundedFrontierReadStop::Limit)
    }

    pub fn bounded_entity_field_equals_for_frontier(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        entities: &BTreeSet<EntityId>,
        kind: KindId,
        locator: &AspectFieldLocator,
        expected: &AspectValue,
        maximum: Option<usize>,
    ) -> Result<
        BoundedFrontierFieldEqualityTruthRead,
        BoundedFrontierReadStop<
            FrontierFieldEqualityTruthReadLimitExceeded,
            crate::execution::RelationalExecutionDenialCause,
        >,
    > {
        self.basis
            .demand_regions_full(request, entities.iter().map(|id| id.partition_id))
            .map_err(|cause| BoundedFrontierReadStop::Admission(cause.into()))?;
        ProjectionRecordReader::new(self.basis.version_id(), &self.instrumentation)
            .bounded_entity_field_equals_for_frontier_in_state(
                &self.basis,
                self.schema(),
                || crate::execution::check_request(request),
                entities,
                kind,
                locator,
                expected,
                self.basis.version_id(),
                maximum,
            )
    }
}
