use super::adjacency_work_ledger::AdjacencyLeaseLedger;
use super::truth_record_access::sort_authoritative_relation_records;
use super::*;
use crate::storage::partition::{AdjacencyDirection, AdjacencyKindBasis};

#[derive(Debug)]
pub struct BoundedAdjacencyTruthRead {
    records: Vec<RelationReadRecord>,
    relation_records_examined: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdjacencyTruthReadLimitExceeded {
    relation_records_examined: usize,
    endpoint_records_reserved: usize,
}

impl VisibilityReadContext<'_> {
    pub fn outgoing_relations_of_kind_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: crate::identity::data::EntityId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        super::super::read_work::metered(
            execution,
            |context| {
                self.relations_of_kind_at_version(
                    entity_id,
                    kind_id,
                    version_id,
                    AdjacencyDirection::Outgoing,
                    &mut |units, _, _| context.checkpoint(units),
                )
                .map(BoundedAdjacencyTruthRead::into_records)
            },
            |answer| {
                answer.as_ref().map_or(0, |records| {
                    super::super::read_footprint::relations(records)
                })
            },
        )?
        .map_err(crate::execution::denial::kernel_stop)
    }
    pub fn bounded_outgoing_relations_of_kind_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: crate::identity::data::EntityId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        maximum_work_units: usize,
    ) -> Result<
        Result<BoundedAdjacencyTruthRead, AdjacencyTruthReadLimitExceeded>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        super::super::read_work::scan(
            execution,
            |context| {
                self.relations_of_kind_at_version(
                    entity_id,
                    kind_id,
                    version_id,
                    AdjacencyDirection::Outgoing,
                    &mut |units, examined, reserved| {
                        if examined + reserved == maximum_work_units {
                            return Err(super::super::native_scan_stop::NativeScanStop::Limit(
                                AdjacencyTruthReadLimitExceeded::new(examined, reserved),
                            ));
                        }
                        context
                            .checkpoint(units)
                            .map_err(super::super::native_scan_stop::NativeScanStop::request)
                    },
                )
            },
            |read| super::super::read_footprint::relations(&read.records),
        )
    }
    pub fn incoming_relations_of_kind_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: crate::identity::data::EntityId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Vec<RelationReadRecord>, crate::execution::RelationalExecutionDenialCause> {
        super::super::read_work::metered(
            execution,
            |context| {
                self.relations_of_kind_at_version(
                    entity_id,
                    kind_id,
                    version_id,
                    AdjacencyDirection::Incoming,
                    &mut |units, _, _| context.checkpoint(units),
                )
                .map(BoundedAdjacencyTruthRead::into_records)
            },
            |answer| {
                answer.as_ref().map_or(0, |records| {
                    super::super::read_footprint::relations(records)
                })
            },
        )?
        .map_err(crate::execution::denial::kernel_stop)
    }
    pub fn bounded_incoming_relations_of_kind_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: crate::identity::data::EntityId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        maximum_work_units: usize,
    ) -> Result<
        Result<BoundedAdjacencyTruthRead, AdjacencyTruthReadLimitExceeded>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        super::super::read_work::scan(
            execution,
            |context| {
                self.relations_of_kind_at_version(
                    entity_id,
                    kind_id,
                    version_id,
                    AdjacencyDirection::Incoming,
                    &mut |units, examined, reserved| {
                        if examined + reserved == maximum_work_units {
                            return Err(super::super::native_scan_stop::NativeScanStop::Limit(
                                AdjacencyTruthReadLimitExceeded::new(examined, reserved),
                            ));
                        }
                        context
                            .checkpoint(units)
                            .map_err(super::super::native_scan_stop::NativeScanStop::request)
                    },
                )
            },
            |read| super::super::read_footprint::relations(&read.records),
        )
    }
    fn relations_of_kind_at_version<E>(
        &self,
        entity_id: crate::identity::data::EntityId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        direction: AdjacencyDirection,
        spend: &mut dyn FnMut(u64, usize, usize) -> Result<(), E>,
    ) -> Result<BoundedAdjacencyTruthRead, E> {
        let edition = self.runtime.acquire_partition_edition();
        let basis =
            AdjacencyKindBasis::of_current_version(version_id == self.runtime.current_version_id());
        let mut ledger = AdjacencyLeaseLedger::default();
        let ids = ledger.lease(
            edition
                .partition(entity_id.partition_id)
                .and_then(|partition| direction.table(partition).get(entity_id.slot_index())),
            basis,
            kind_id,
        );
        let answer = (|| {
            let mut records = Vec::new();
            let mut examined = 0;
            for id in ids.iter().copied() {
                spend(
                    super::super::read_prices::EXAMINE_SLOT,
                    examined,
                    records.len(),
                )?;
                examined += 1;
                let Some(record) =
                    self.authoritative_relation_record_for_id_at_version(&edition, id, version_id)
                else {
                    continue;
                };
                if record.kind.kind_id != kind_id
                    || record.lifecycle != crate::storage::data::RecordLifecycleState::Live
                    || !direction.matches_endpoint(&record, entity_id)
                {
                    continue;
                }
                spend(
                    super::super::read_prices::RESERVE_RECORD,
                    examined,
                    records.len(),
                )?;
                records.push(record);
            }
            sort_authoritative_relation_records(&mut records);
            Ok(BoundedAdjacencyTruthRead::new(records, examined))
        })();
        ledger.settle(self.runtime);
        answer
    }
}

impl BoundedAdjacencyTruthRead {
    pub(crate) fn owned_bytes(&self) -> u64 {
        super::super::read_footprint::relations(&self.records)
    }
    pub(crate) const fn new(
        records: Vec<RelationReadRecord>,
        relation_records_examined: usize,
    ) -> Self {
        Self {
            records,
            relation_records_examined,
        }
    }

    pub const fn relation_records_examined(&self) -> usize {
        self.relation_records_examined
    }

    pub const fn endpoint_records_reserved(&self) -> usize {
        self.records.len()
    }

    pub const fn work_units(&self) -> usize {
        self.relation_records_examined + self.records.len()
    }

    pub fn into_records(self) -> Vec<RelationReadRecord> {
        self.records
    }
}

impl AdjacencyTruthReadLimitExceeded {
    pub(crate) const fn new(
        relation_records_examined: usize,
        endpoint_records_reserved: usize,
    ) -> Self {
        Self {
            relation_records_examined,
            endpoint_records_reserved,
        }
    }

    pub const fn relation_records_examined(self) -> usize {
        self.relation_records_examined
    }

    pub const fn endpoint_records_reserved(self) -> usize {
        self.endpoint_records_reserved
    }

    pub const fn consumed_work_units(self) -> usize {
        self.relation_records_examined + self.endpoint_records_reserved
    }
}
