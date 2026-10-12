//! Frontier work uses one captured exact root and preserves caller interruption.
use super::super::*;
use super::*;
use crate::identity::data::{EntityId, KindId};
use crate::storage::partition::AdjacencyDirection;
use std::collections::BTreeSet;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
impl RelationalPinnedRead {
    pub fn bounded_outgoing_relations_for_frontier(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        entities: &BTreeSet<EntityId>,
        kind: KindId,
        maximum: usize,
    ) -> Result<
        BoundedFrontierAdjacencyTruthRead,
        BoundedFrontierReadStop<
            FrontierAdjacencyTruthReadLimitExceeded,
            worth_execution::MapKernelStop,
        >,
    > {
        self.frontier_relations(
            request,
            entities,
            kind,
            maximum,
            AdjacencyDirection::Outgoing,
        )
    }
    pub fn bounded_incoming_relations_for_frontier(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        entities: &BTreeSet<EntityId>,
        kind: KindId,
        maximum: usize,
    ) -> Result<
        BoundedFrontierAdjacencyTruthRead,
        BoundedFrontierReadStop<
            FrontierAdjacencyTruthReadLimitExceeded,
            worth_execution::MapKernelStop,
        >,
    > {
        self.frontier_relations(
            request,
            entities,
            kind,
            maximum,
            AdjacencyDirection::Incoming,
        )
    }
    fn frontier_relations(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        entities: &BTreeSet<EntityId>,
        kind: KindId,
        maximum: usize,
        direction: AdjacencyDirection,
    ) -> Result<
        BoundedFrontierAdjacencyTruthRead,
        BoundedFrontierReadStop<
            FrontierAdjacencyTruthReadLimitExceeded,
            worth_execution::MapKernelStop,
        >,
    > {
        ProjectionRecordReader::new(self.current_version, &self.instrumentation)
            .bounded_relations_for_frontier_in_state(
                self.basis.root().as_ref(),
                self.schema(),
                || request.check_live(),
                entities,
                kind,
                self.basis.version_id(),
                direction,
                maximum,
            )
    }
    pub fn bounded_entity_field_equals_for_frontier(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        entities: &BTreeSet<EntityId>,
        kind: KindId,
        locator: &AspectFieldLocator,
        expected: &AspectValue,
        maximum: usize,
    ) -> Result<
        BoundedFrontierFieldEqualityTruthRead,
        BoundedFrontierReadStop<
            FrontierFieldEqualityTruthReadLimitExceeded,
            worth_execution::MapKernelStop,
        >,
    > {
        ProjectionRecordReader::new(self.current_version, &self.instrumentation)
            .bounded_entity_field_equals_for_frontier_in_state(
                self.basis.root().as_ref(),
                self.schema(),
                || request.check_live(),
                entities,
                kind,
                locator,
                expected,
                self.basis.version_id(),
                maximum,
            )
    }
}
