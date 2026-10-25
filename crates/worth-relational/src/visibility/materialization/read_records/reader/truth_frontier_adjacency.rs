use std::collections::BTreeSet;

use super::adjacency_work_ledger::AdjacencyLeaseLedger;
use super::truth_record_access::sort_authoritative_relation_records;
use super::*;
use crate::storage::partition::{AdjacencyDirection, AdjacencyKindBasis};

#[derive(Debug)]
pub struct BoundedFrontierAdjacencyTruthRead {
    pub(crate) records: Vec<RelationReadRecord>,
    pub(crate) adjacency_lists_read: usize,
    pub(crate) relation_records_examined: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrontierAdjacencyTruthReadLimitExceeded {
    adjacency_lists_read: usize,
    relation_records_examined: usize,
    endpoint_records_reserved: usize,
}

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
        crate::execution::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.acquire_partition_edition();
        super::super::read_work::with_full_residency(execution, &edition, |state| {
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
        })
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
        crate::execution::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.acquire_partition_edition();
        super::super::read_work::with_full_residency(execution, &edition, |state| {
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
        })
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

impl BoundedFrontierAdjacencyTruthRead {
    pub const fn adjacency_lists_read(&self) -> usize {
        self.adjacency_lists_read
    }

    pub const fn relation_records_examined(&self) -> usize {
        self.relation_records_examined
    }

    pub const fn endpoint_records_reserved(&self) -> usize {
        self.records.len()
    }

    pub const fn work_units(&self) -> usize {
        self.adjacency_lists_read
            .saturating_add(self.relation_records_examined)
            .saturating_add(self.records.len())
    }

    pub fn into_records(self) -> Vec<RelationReadRecord> {
        self.records
    }
}

impl FrontierAdjacencyTruthReadLimitExceeded {
    pub(crate) const fn new(
        adjacency_lists_read: usize,
        relation_records_examined: usize,
        endpoint_records_reserved: usize,
    ) -> Self {
        Self {
            adjacency_lists_read,
            relation_records_examined,
            endpoint_records_reserved,
        }
    }

    pub const fn adjacency_lists_read(self) -> usize {
        self.adjacency_lists_read
    }

    pub const fn relation_records_examined(self) -> usize {
        self.relation_records_examined
    }

    pub const fn endpoint_records_reserved(self) -> usize {
        self.endpoint_records_reserved
    }

    pub const fn consumed_work_units(self) -> usize {
        self.adjacency_lists_read
            .saturating_add(self.relation_records_examined)
            .saturating_add(self.endpoint_records_reserved)
    }
}

fn charge_frontier_work(
    maximum_work_units: Option<usize>,
    adjacency_lists_read: usize,
    relation_records_examined: usize,
    endpoint_records_reserved: usize,
) -> Result<(), FrontierAdjacencyTruthReadLimitExceeded> {
    if maximum_work_units.is_some_and(|maximum| {
        adjacency_lists_read
            .saturating_add(relation_records_examined)
            .saturating_add(endpoint_records_reserved)
            >= maximum
    }) {
        Err(FrontierAdjacencyTruthReadLimitExceeded::new(
            adjacency_lists_read,
            relation_records_examined,
            endpoint_records_reserved,
        ))
    } else {
        Ok(())
    }
}

impl ProjectionRecordReader<'_> {
    pub(crate) fn bounded_relations_for_frontier_in_state<E>(
        &self,
        edition: &(impl PartitionAccess + ?Sized),
        registry: &crate::schema::data::RelationalSchemaRegistry,
        mut check_live: impl FnMut() -> Result<(), E>,
        entity_ids: &BTreeSet<crate::identity::data::EntityId>,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        direction: AdjacencyDirection,
        maximum_work_units: Option<usize>,
        include_audit: bool,
    ) -> Result<
        BoundedFrontierAdjacencyTruthRead,
        BoundedFrontierReadStop<FrontierAdjacencyTruthReadLimitExceeded, E>,
    > {
        let basis = AdjacencyKindBasis::of_current_version(version_id == self.current_version);
        // One edition for the whole frontier. A width-W frontier of degree D
        // costs one substrate acquisition, not W of them, and every entity in
        // the frontier is expanded against the same substrate.
        let mut ledger = AdjacencyLeaseLedger::default();
        let mut records = Vec::new();
        let mut adjacency_lists_read = 0_usize;
        let mut relation_records_examined = 0_usize;
        for entity_id in entity_ids {
            if let Err(exceeded) = charge_frontier_work(
                maximum_work_units,
                adjacency_lists_read,
                relation_records_examined,
                records.len(),
            ) {
                ledger.settle_read_round(edition, self.instrumentation);
                return Err(BoundedFrontierReadStop::Limit(exceeded));
            }
            if let Err(stop) = check_live() {
                ledger.settle_read_round(edition, self.instrumentation);
                return Err(BoundedFrontierReadStop::Admission(stop));
            }
            adjacency_lists_read += 1;
            // Leased per frontier entity and never copied, so an entity whose
            // fanout dwarfs the remaining budget still costs only the units
            // actually spent walking it.
            let relation_ids = ledger.lease(
                edition
                    .get_partition(entity_id.partition_id)
                    .and_then(|partition| direction.table(partition).get(entity_id.slot_index())),
                basis,
                kind_id,
            );
            for relation_id in relation_ids.iter().copied() {
                if let Err(exceeded) = charge_frontier_work(
                    maximum_work_units,
                    adjacency_lists_read,
                    relation_records_examined,
                    records.len(),
                ) {
                    ledger.settle_read_round(edition, self.instrumentation);
                    return Err(BoundedFrontierReadStop::Limit(exceeded));
                }
                if let Err(stop) = check_live() {
                    ledger.settle_read_round(edition, self.instrumentation);
                    return Err(BoundedFrontierReadStop::Admission(stop));
                }
                relation_records_examined += 1;
                let Some(record) = self
                    .authoritative_relation_record_for_id_at_version_with_registry(
                        edition,
                        registry,
                        relation_id,
                        version_id,
                    )
                else {
                    continue;
                };
                if record.kind.kind_id != kind_id
                    || !(record.lifecycle == crate::storage::data::RecordLifecycleState::Live
                        || (include_audit && record.lifecycle == crate::storage::data::RecordLifecycleState::RetainedDanglingForAudit))
                    || !direction.matches_endpoint(&record, *entity_id)
                {
                    continue;
                }
                if let Err(exceeded) = charge_frontier_work(
                    maximum_work_units,
                    adjacency_lists_read,
                    relation_records_examined,
                    records.len(),
                ) {
                    ledger.settle_read_round(edition, self.instrumentation);
                    return Err(BoundedFrontierReadStop::Limit(exceeded));
                }
                if let Err(stop) = check_live() {
                    ledger.settle_read_round(edition, self.instrumentation);
                    return Err(BoundedFrontierReadStop::Admission(stop));
                }
                records.push(record);
            }
        }
        ledger.settle_read_round(edition, self.instrumentation);
        sort_authoritative_relation_records(&mut records);
        Ok(BoundedFrontierAdjacencyTruthRead {
            records,
            adjacency_lists_read,
            relation_records_examined,
        })
    }
}
