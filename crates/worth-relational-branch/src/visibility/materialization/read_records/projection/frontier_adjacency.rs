use std::collections::BTreeSet;
use worth_relational_contracts::internal_port::execution::request_or::RequestOr;

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
        use super::super::read_prices::{EXAMINE_SLOT, RESERVE_RECORD};
        use super::super::read_work::record_memory;

        let settled = super::super::read_work::scan_with_retained_memory(
            execution,
            |context, memory| {
                let Some(root) = self.basis.root() else {
                    let mut lists = 0;
                    for _ in entity_ids {
                        charge(maximum_work_units, lists, 0, 0).map_err(RequestOr::Inner)?;
                        context
                            .checkpoint(EXAMINE_SLOT)
                            .map_err(RequestOr::from_kernel_stop)?;
                        lists += 1;
                    }
                    return Ok(BoundedFrontierAdjacencyTruthRead {
                        records: Vec::new(),
                        adjacency_lists_read: lists,
                        relation_records_examined: 0,
                        memory: None,
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
                            .map_err(RequestOr::Inner)?;
                        context
                            .checkpoint(EXAMINE_SLOT)
                            .map_err(RequestOr::from_kernel_stop)?;
                        lists += 1;
                        let relation_ids = if self.is_exact_basis() {
                            let set = if let Some(head) =
                                self.admitted_head_region(entity_id.partition_id)
                            {
                                let table = head.adjacency(direction);
                                context
                                    .checkpoint(table.navigation_work_bound() as u64)
                                    .map_err(RequestOr::from_kernel_stop)?;
                                let set = table.get(entity_id.slot_index());
                                if let Some(set) = set {
                                    context
                                        .checkpoint(set.navigation_work_bound() as u64)
                                        .map_err(RequestOr::from_kernel_stop)?;
                                }
                                set
                            } else {
                                None
                            };
                            ledger.lease_current(set, kind_id)
                        } else {
                            let set = if let Some(partition) =
                                root.get_partition(entity_id.partition_id)
                            {
                                let table = direction.table(partition);
                                context
                                    .checkpoint(table.navigation_work_bound() as u64)
                                    .map_err(RequestOr::from_kernel_stop)?;
                                let set = table.get(entity_id.slot_index());
                                if let Some(set) = set {
                                    context
                                        .checkpoint(
                                            set.selected_kind_navigation_work_bound(adjacency_basis)
                                                as u64,
                                        )
                                        .map_err(RequestOr::from_kernel_stop)?;
                                }
                                set
                            } else {
                                None
                            };
                            ledger.lease(set, adjacency_basis, kind_id)
                        };
                        let mut iterator_bytes = 0_u64;
                        let mut iterator = relation_ids.try_iter(|navigation, bytes| {
                            charge_iter_step(
                                context,
                                memory,
                                &mut iterator_bytes,
                                navigation,
                                bytes,
                            )
                        })?;
                        while let Some(&relation_id) = iterator.try_next(|navigation, bytes| {
                            charge_iter_step(
                                context,
                                memory,
                                &mut iterator_bytes,
                                navigation,
                                bytes,
                            )
                        })? {
                            charge(maximum_work_units, lists, examined, records.len())
                                .map_err(RequestOr::Inner)?;
                            context
                                .checkpoint(EXAMINE_SLOT)
                                .map_err(RequestOr::from_kernel_stop)?;
                            examined += 1;
                            if !self.relation_matches_frontier(
                                &root,
                                relation_id,
                                *entity_id,
                                kind_id,
                                direction,
                                include_audit,
                            ) {
                                continue;
                            }
                            charge(maximum_work_units, lists, examined, records.len())
                                .map_err(RequestOr::Inner)?;
                            context
                                .checkpoint(RESERVE_RECORD)
                                .map_err(RequestOr::from_kernel_stop)?;
                            let clone_bytes =
                                super::ProjectionRecordSource::candidate_relation_heap_for_id(
                                    self,
                                    relation_id,
                                )
                                .map_err(RequestOr::Request)?;
                            record_memory::prepare_push(memory, &mut records, clone_bytes)
                                .map_err(RequestOr::Request)?;
                            let Some(record) = self.project_relation_record(relation_id) else {
                                record_memory::release(memory, clone_bytes)
                                    .map_err(RequestOr::Request)?;
                                continue;
                            };
                            let actual = super::super::read_footprint::relation(&record);
                            record_memory::settle_clone(memory, clone_bytes, actual)
                                .map_err(RequestOr::Request)?;
                            records.push(record);
                        }
                        record_memory::release(memory, iterator_bytes)
                            .map_err(RequestOr::Request)?;
                    }
                    let sort_work = sort_work_units(records.len())
                        .ok_or(RequestOr::Request(
                            crate::execution::denial_cause::RelationalExecutionDenialCause::WorkCounterOverflow,
                        ))?;
                    if sort_work > 0 {
                        context
                            .checkpoint(sort_work)
                            .map_err(RequestOr::from_kernel_stop)?;
                    }
                    records.sort_unstable_by_key(relation_identity_order_key);
                    Ok(BoundedFrontierAdjacencyTruthRead {
                        records,
                        adjacency_lists_read: lists,
                        relation_records_examined: examined,
                        memory: None,
                    })
                })();
                ledger.settle_instrumentation(&self.instrumentation);
                answer
            },
            |read| read.owned_bytes(),
        )?;
        Ok(settled.map(|(mut read, reservation)| {
            read.memory = Some(reservation);
            read
        }))
    }
    fn relation_matches_frontier(
        &self,
        root: &crate::branch::root::projection::SnapshotProjectionRoot<'_>,
        relation_id: crate::identity::data::RelationId,
        anchor: EntityId,
        kind_id: KindId,
        direction: AdjacencyDirection,
        include_audit: bool,
    ) -> bool {
        let demand = super::adjacency_demand::AdjacencyDemand {
            kind: kind_id,
            direction,
            maximum: None,
            frontier: true,
            include_audit,
        };
        let schema = root.schema_authority().registry();
        if self.is_exact_basis() {
            self.admitted_head_region(relation_id.partition_id)
                .is_some_and(|head| demand.current_matches(schema, head, relation_id, anchor))
        } else {
            root.get_partition(relation_id.partition_id)
                .is_some_and(|partition| {
                    demand.historical_matches(
                        schema,
                        partition,
                        relation_id,
                        anchor,
                        self.version_id(),
                    )
                })
        }
    }
}

fn charge_iter_step(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
    iterator_bytes: &mut u64,
    navigation: u64,
    bytes: u64,
) -> Result<(), RequestOr<FrontierAdjacencyTruthReadLimitExceeded>> {
    if navigation > 0 {
        context
            .checkpoint(navigation)
            .map_err(RequestOr::from_kernel_stop)?;
    }
    if bytes > 0 {
        super::super::read_work::record_memory::claim(memory, bytes).map_err(RequestOr::Request)?;
        *iterator_bytes = iterator_bytes.checked_add(bytes).ok_or(RequestOr::Request(
            crate::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
        ))?;
    }
    Ok(())
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
