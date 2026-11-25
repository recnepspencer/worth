//! Frontier work uses one captured exact root and preserves caller interruption.
use super::RelationalPinnedRead;
use branch::visibility::materialization::read_records::projection::frontier_adjacency::VisibilityProjectionViewOwnerPort as _;
use branch::visibility::materialization::read_records::reader::frontier_admission::BoundedFrontierReadStop;
use branch::visibility::materialization::read_records::reader::projection_record_reader::ProjectionRecordReader;
use branch::visibility::materialization::read_records::reader::truth_frontier_adjacency::BoundedFrontierAdjacencyTruthRead;
use branch::visibility::materialization::read_records::reader::truth_frontier_adjacency::FrontierAdjacencyTruthReadLimitExceeded;
use branch::visibility::materialization::read_records::reader::truth_frontier_field_equality::BoundedFrontierFieldEqualityTruthRead;
use branch::visibility::materialization::read_records::reader::truth_frontier_field_equality::FrontierFieldEqualityTruthReadLimitExceeded;
use contracts::identity::data::primitives::KindId;
use contracts::identity::data::record_ids::EntityId;
use std::collections::BTreeSet;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::partition::adjacency_direction::AdjacencyDirection;
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
            contracts::execution::denial_cause::RelationalExecutionDenialCause,
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
            contracts::execution::denial_cause::RelationalExecutionDenialCause,
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
            contracts::execution::denial_cause::RelationalExecutionDenialCause,
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
            contracts::execution::denial_cause::RelationalExecutionDenialCause,
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
            contracts::execution::denial_cause::RelationalExecutionDenialCause,
        >,
    > {
        self.basis
            .demand_regions_full(request, entities.iter().map(|id| id.partition_id))
            .map_err(|cause| BoundedFrontierReadStop::Admission(cause.into()))?;
        ProjectionRecordReader::new(self.basis.version_id(), &self.instrumentation)
            .bounded_entity_field_equals_for_frontier_in_state(
                &self.basis,
                self.schema(),
                || contracts::execution::request_custody::check_request(request),
                entities,
                kind,
                locator,
                expected,
                self.basis.version_id(),
                maximum,
            )
    }
}
