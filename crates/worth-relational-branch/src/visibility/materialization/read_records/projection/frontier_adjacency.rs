use std::collections::BTreeSet;

use crate::identity::data::{EntityId, KindId};
use crate::storage::overlay::PartitionAccess;
use crate::storage::partition::{AdjacencyDirection, AdjacencyKindBasis};

use super::read_record_identity_ordering::relation_identity_order_key;
use super::VisibilityProjectionView;
use crate::visibility::materialization::read_records::reader::adjacency_work_ledger::AdjacencyLeaseLedger;
use crate::visibility::materialization::read_records::reader::truth_frontier_adjacency::BoundedFrontierAdjacencyTruthRead;
use crate::visibility::materialization::read_records::reader::truth_frontier_adjacency::FrontierAdjacencyTruthReadLimitExceeded;

impl VisibilityProjectionView {
    pub fn bounded_outgoing_relations_for_frontier(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_ids: &BTreeSet<EntityId>,
        kind_id: KindId,
        maximum_work_units: Option<usize>,
    ) -> Result<
        Result<BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.bounded_relations_for_frontier(
            execution,
            entity_ids,
            kind_id,
            AdjacencyDirection::Outgoing,
            maximum_work_units,
            false,
        )
    }

    pub fn bounded_incoming_relations_for_frontier(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_ids: &BTreeSet<EntityId>,
        kind_id: KindId,
        maximum_work_units: Option<usize>,
    ) -> Result<
        Result<BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.bounded_relations_for_frontier(
            execution,
            entity_ids,
            kind_id,
            AdjacencyDirection::Incoming,
            maximum_work_units,
            false,
        )
    }

    /// Index maintenance mirrors full-build relation visibility, including
    /// audit-retained edges. Ordinary frontier queries remain Live-only.
    pub fn bounded_index_relations_for_frontier(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_ids: &BTreeSet<EntityId>,
        kind_id: KindId,
        outgoing: bool,
        maximum_work_units: Option<usize>,
    ) -> Result<
        Result<BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.bounded_relations_for_frontier(
            execution,
            entity_ids,
            kind_id,
            if outgoing {
                AdjacencyDirection::Outgoing
            } else {
                AdjacencyDirection::Incoming
            },
            maximum_work_units,
            true,
        )
    }

    pub(crate) fn bounded_relations_for_frontier(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_ids: &BTreeSet<EntityId>,
        kind_id: KindId,
        direction: AdjacencyDirection,
        maximum_work_units: Option<usize>,
        include_audit: bool,
    ) -> Result<
        Result<BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.demand_adjacency(
            execution,
            entity_ids.iter().copied(),
            super::adjacency_demand::AdjacencyDemand {
                kind: kind_id,
                direction,
                maximum: maximum_work_units,
                frontier: true,
                include_audit,
            },
        )?;
        use super::super::native_scan_stop::NativeScanStop;
        use super::super::read_prices::{EXAMINE_SLOT, RESERVE_RECORD};
        use super::super::read_work::record_memory;

        super::super::read_work::scan_with_memory(
            execution,
            |context, memory| {
                let Some(root) = self.basis.root() else {
                    let mut lists = 0;
                    for _ in entity_ids {
                        charge(maximum_work_units, lists, 0, 0).map_err(NativeScanStop::Limit)?;
                        context
                            .checkpoint(EXAMINE_SLOT)
                            .map_err(NativeScanStop::request)?;
                        lists += 1;
                    }
                    return Ok(BoundedFrontierAdjacencyTruthRead {
                        records: Vec::new(),
                        adjacency_lists_read: lists,
                        relation_records_examined: 0,
                    });
                };
                let adjacency_basis = AdjacencyKindBasis::of_current_version(
                    self.basis.root_version() == Some(self.version_id()),
                );
                let mut ledger = AdjacencyLeaseLedger::default();
                let answer = (|| {
                    let mut records = Vec::new();
                    let mut lists = 0_usize;
                    let mut examined = 0_usize;
                    for entity_id in entity_ids {
                        charge(maximum_work_units, lists, examined, records.len())
                            .map_err(NativeScanStop::Limit)?;
                        context
                            .checkpoint(EXAMINE_SLOT)
                            .map_err(NativeScanStop::request)?;
                        lists += 1;
                        let relation_ids = if self.is_exact_basis() {
                            ledger.lease_current(
                                self.admitted_head_region(entity_id.partition_id).and_then(
                                    |head| head.adjacency(direction).get(entity_id.slot_index()),
                                ),
                                kind_id,
                            )
                        } else {
                            ledger.lease(
                                root.get_partition(entity_id.partition_id)
                                    .and_then(|partition| {
                                        direction.table(partition).get(entity_id.slot_index())
                                    }),
                                adjacency_basis,
                                kind_id,
                            )
                        };
                        let mut iterator_bytes = 0_u64;
                        let iterator = relation_ids.try_iter(|_navigation, bytes| {
                            record_memory::claim(memory, bytes).map_err(NativeScanStop::Request)?;
                            iterator_bytes = bytes;
                            Ok(())
                        })?;
                        for relation_id in iterator.copied() {
                            charge(maximum_work_units, lists, examined, records.len())
                                .map_err(NativeScanStop::Limit)?;
                            context
                                .checkpoint(EXAMINE_SLOT)
                                .map_err(NativeScanStop::request)?;
                            examined += 1;
                            let clone_bytes =
                                super::ProjectionRecordSource::candidate_relation_heap_for_id(
                                    self,
                                    relation_id,
                                )
                                .map_err(NativeScanStop::Request)?;
                            record_memory::claim(memory, clone_bytes)
                                .map_err(NativeScanStop::Request)?;
                            let Some(record) = self.project_relation_record(relation_id) else {
                                record_memory::release(memory, clone_bytes)
                                    .map_err(NativeScanStop::Request)?;
                                continue;
                            };
                            let actual = super::super::read_footprint::relation(&record);
                            record_memory::settle_clone(memory, clone_bytes, actual)
                                .map_err(NativeScanStop::Request)?;
                            if record.kind.kind_id != kind_id
                                || !(record.lifecycle
                                    == crate::storage::data::RecordLifecycleState::Live
                                    || (include_audit
                                        && record.lifecycle
                                            == crate::storage::data::RecordLifecycleState::RetainedDanglingForAudit))
                                || !direction.matches_endpoint(&record, *entity_id)
                            {
                                drop(record);
                                record_memory::release(memory, actual)
                                    .map_err(NativeScanStop::Request)?;
                                continue;
                            }
                            charge(maximum_work_units, lists, examined, records.len())
                                .map_err(NativeScanStop::Limit)?;
                            context
                                .checkpoint(RESERVE_RECORD)
                                .map_err(NativeScanStop::request)?;
                            record_memory::prepare_push(memory, &mut records, 0)
                                .map_err(NativeScanStop::Request)?;
                            records.push(record);
                        }
                        record_memory::release(memory, iterator_bytes)
                            .map_err(NativeScanStop::Request)?;
                    }
                    let sort_work = sort_work_units(records.len())
                        .ok_or(NativeScanStop::Request(
                            crate::execution::denial_cause::RelationalExecutionDenialCause::WorkCounterOverflow,
                        ))?;
                    if sort_work > 0 {
                        context
                            .checkpoint(sort_work)
                            .map_err(NativeScanStop::request)?;
                    }
                    records.sort_unstable_by_key(relation_identity_order_key);
                    Ok(BoundedFrontierAdjacencyTruthRead {
                        records,
                        adjacency_lists_read: lists,
                        relation_records_examined: examined,
                    })
                })();
                ledger.settle_instrumentation(&self.instrumentation);
                answer
            },
            |read| read.owned_bytes(),
        )
    }
}

fn sort_work_units(len: usize) -> Option<u64> {
    if len <= 1 {
        return Some(0);
    }
    let log2 = u64::from(usize::BITS - (len - 1).leading_zeros());
    u64::try_from(len).ok()?.checked_mul(log2)
}

fn charge(
    maximum: Option<usize>,
    lists: usize,
    examined: usize,
    endpoints: usize,
) -> Result<(), FrontierAdjacencyTruthReadLimitExceeded> {
    if maximum
        .is_some_and(|maximum| lists.saturating_add(examined).saturating_add(endpoints) >= maximum)
    {
        Err(FrontierAdjacencyTruthReadLimitExceeded::new(
            lists, examined, endpoints,
        ))
    } else {
        Ok(())
    }
}

#[doc(hidden)]
pub trait VisibilityProjectionViewOwnerPort: Sized {
    fn bounded_relations_for_frontier(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_ids: &BTreeSet<EntityId>,
        kind_id: KindId,
        direction: AdjacencyDirection,
        maximum_work_units: Option<usize>,
        include_audit: bool,
    ) -> Result<
        Result<BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    >;
}

impl VisibilityProjectionViewOwnerPort for VisibilityProjectionView {
    fn bounded_relations_for_frontier(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_ids: &BTreeSet<EntityId>,
        kind_id: KindId,
        direction: AdjacencyDirection,
        maximum_work_units: Option<usize>,
        include_audit: bool,
    ) -> Result<
        Result<BoundedFrontierAdjacencyTruthRead, FrontierAdjacencyTruthReadLimitExceeded>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.bounded_relations_for_frontier(
            execution,
            entity_ids,
            kind_id,
            direction,
            maximum_work_units,
            include_audit,
        )
    }
}
