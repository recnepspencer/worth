//! Select the installed read operands before handing off to the read implementation.
use super::*;
use crate::visibility::materialization::read_records::reader::context::VisibilityReadContext;

impl<'runtime> VisibilityReadContext<'runtime> {
    pub fn bounded_outgoing_relations_for_frontier_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_ids: &BTreeSet<crate::identity::data::EntityId>,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        maximum_work_units: usize,
    ) -> Result<
        Result<BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.acquire_partition_edition();
        super::super::super::read_work::with_full_residency(
            execution,
            &edition,
            &self.runtime.services.symbols,
            |state| {
                crate::visibility::materialization::read_records::read_request::checked(
                    execution,
                    || {
                        self.bounded_relations_for_frontier_at_version(
                            state,
                            entity_ids,
                            kind_id,
                            version_id,
                            AdjacencyDirection::Outgoing,
                            maximum_work_units,
                        )
                    },
                )
            },
        )
    }

    pub fn bounded_incoming_relations_for_frontier_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_ids: &BTreeSet<crate::identity::data::EntityId>,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        maximum_work_units: usize,
    ) -> Result<
        Result<BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.acquire_partition_edition();
        super::super::super::read_work::with_full_residency(
            execution,
            &edition,
            &self.runtime.services.symbols,
            |state| {
                crate::visibility::materialization::read_records::read_request::checked(
                    execution,
                    || {
                        self.bounded_relations_for_frontier_at_version(
                            state,
                            entity_ids,
                            kind_id,
                            version_id,
                            AdjacencyDirection::Incoming,
                            maximum_work_units,
                        )
                    },
                )
            },
        )
    }

    fn bounded_relations_for_frontier_at_version(
        &self,
        edition: &dyn PartitionAccess,
        entity_ids: &BTreeSet<crate::identity::data::EntityId>,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        direction: AdjacencyDirection,
        maximum_work_units: usize,
    ) -> Result<BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded> {
        ProjectionRecordReader::new(
            self.runtime.current_version_id(),
            &self.runtime.services.instrumentation,
        )
        .bounded_relations_for_frontier_in_state(
            edition,
            &self.runtime.config.schema.registry,
            || Ok::<_, std::convert::Infallible>(()),
            entity_ids,
            kind_id,
            version_id,
            direction,
            Some(maximum_work_units),
            false,
        )
        .map_err(|stop| match stop {
            BoundedFrontierReadStop::Limit(limit) => limit,
            BoundedFrontierReadStop::Admission(never) => match never {},
        })
    }
}
