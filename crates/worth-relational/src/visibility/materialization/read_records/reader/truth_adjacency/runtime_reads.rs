//! Select the installed read operands before handing off to the read implementation.
use worth_relational_branch::internal_port::visibility::materialization::read_records::reader::truth_adjacency::BoundedAdjacencyTruthReadFieldsPort as _;
use worth_relational_branch::internal_port::visibility::materialization::read_records::reader::truth_adjacency::AdjacencyTruthReadLimitExceededOwnerPort as _;
use worth_relational_branch::internal_port::visibility::materialization::read_records::reader::truth_adjacency::BoundedAdjacencyTruthReadOwnerPort as _;
use super::*;
use crate::visibility::materialization::read_records::reader::context::VisibilityReadContext;

impl VisibilityReadContext<'_> {
    pub fn outgoing_relations_of_kind_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: crate::identity::data::EntityId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.acquire_partition_edition();
        super::super::super::read_work::with_full_residency(
            execution,
            &edition,
            &self.runtime.services.symbols,
            |state| {
                super::super::super::read_work::metered_with_memory(
                    execution,
                    |context, memory| {
                        self.relations_of_kind_at_version(
                            state,
                            entity_id,
                            kind_id,
                            version_id,
                            AdjacencyDirection::Outgoing,
                            &mut |units, _, _| {
                                context
                                    .checkpoint(units)
                                    .map_err(crate::execution::denial::kernel_stop)
                            },
                            memory,
                            &std::convert::identity,
                        )
                        .map(BoundedAdjacencyTruthRead::into_records)
                    },
                    |answer| {
                        answer.as_ref().map_or(0, |records| {
                            super::super::super::read_footprint::relations(records)
                        })
                    },
                )?
            },
        )
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
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.acquire_partition_edition();
        super::super::super::read_work::with_full_residency(
            execution,
            &edition,
            &self.runtime.services.symbols,
            |state| {
                super::super::super::read_work::scan_with_memory(
                    execution,
                    |context, memory| {
                        self.relations_of_kind_at_version(
                            state,
                            entity_id,
                            kind_id,
                            version_id,
                            AdjacencyDirection::Outgoing,
                            &mut |units, examined, reserved| {
                                if examined + reserved == maximum_work_units {
                                    return Err(
                                        super::super::super::native_scan_stop::NativeScanStop::Limit(
                                            AdjacencyTruthReadLimitExceeded::new(
                                                examined, reserved,
                                            ),
                                        ),
                                    );
                                }
                                context.checkpoint(units).map_err(
                                    super::super::super::native_scan_stop::NativeScanStop::request,
                                )
                            },
                            memory,
                            &super::super::super::native_scan_stop::NativeScanStop::Request,
                        )
                    },
                    |read| super::super::super::read_footprint::relations(&(*read.owner_records())),
                )
            },
        )
    }
    pub fn incoming_relations_of_kind_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: crate::identity::data::EntityId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.acquire_partition_edition();
        super::super::super::read_work::with_full_residency(
            execution,
            &edition,
            &self.runtime.services.symbols,
            |state| {
                super::super::super::read_work::metered_with_memory(
                    execution,
                    |context, memory| {
                        self.relations_of_kind_at_version(
                            state,
                            entity_id,
                            kind_id,
                            version_id,
                            AdjacencyDirection::Incoming,
                            &mut |units, _, _| {
                                context
                                    .checkpoint(units)
                                    .map_err(crate::execution::denial::kernel_stop)
                            },
                            memory,
                            &std::convert::identity,
                        )
                        .map(BoundedAdjacencyTruthRead::into_records)
                    },
                    |answer| {
                        answer.as_ref().map_or(0, |records| {
                            super::super::super::read_footprint::relations(records)
                        })
                    },
                )?
            },
        )
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
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let edition = self.runtime.acquire_partition_edition();
        super::super::super::read_work::with_full_residency(
            execution,
            &edition,
            &self.runtime.services.symbols,
            |state| {
                super::super::super::read_work::scan_with_memory(
                    execution,
                    |context, memory| {
                        self.relations_of_kind_at_version(
                            state,
                            entity_id,
                            kind_id,
                            version_id,
                            AdjacencyDirection::Incoming,
                            &mut |units, examined, reserved| {
                                if examined + reserved == maximum_work_units {
                                    return Err(
                                        super::super::super::native_scan_stop::NativeScanStop::Limit(
                                            AdjacencyTruthReadLimitExceeded::new(
                                                examined, reserved,
                                            ),
                                        ),
                                    );
                                }
                                context.checkpoint(units).map_err(
                                    super::super::super::native_scan_stop::NativeScanStop::request,
                                )
                            },
                            memory,
                            &super::super::super::native_scan_stop::NativeScanStop::Request,
                        )
                    },
                    |read| super::super::super::read_footprint::relations(&(*read.owner_records())),
                )
            },
        )
    }
    fn relations_of_kind_at_version<E>(
        &self,
        edition: &dyn PartitionAccess,
        entity_id: crate::identity::data::EntityId,
        kind_id: crate::identity::data::KindId,
        version_id: crate::identity::data::VersionId,
        direction: AdjacencyDirection,
        spend: &mut dyn FnMut(u64, usize, usize) -> Result<(), E>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
        request: &dyn Fn(crate::execution::denial_cause::RelationalExecutionDenialCause) -> E,
    ) -> Result<BoundedAdjacencyTruthRead, E> {
        let basis =
            AdjacencyKindBasis::of_current_version(version_id == self.runtime.current_version_id());
        let mut ledger = AdjacencyLeaseLedger::default();
        let ids = ledger.lease(
            edition
                .get_partition(entity_id.partition_id)
                .and_then(|partition| direction.table(partition).get(entity_id.slot_index())),
            basis,
            kind_id,
        );
        let answer = (|| {
            let mut records = Vec::new();
            let mut examined = 0;
            let mut iterator_bytes = 0;
            let ids = ids.try_iter(|_navigation, bytes| {
                super::super::super::read_work::record_memory::claim(memory, bytes)
                    .map_err(request)?;
                iterator_bytes = bytes;
                Ok(())
            })?;
            for id in ids.copied() {
                spend(
                    super::super::super::read_prices::EXAMINE_SLOT,
                    examined,
                    records.len(),
                )?;
                examined += 1;
                let clone_bytes = super::super::super::read_work::record_quote::relation_for_id(
                    edition,
                    &self.runtime.config.schema.registry,
                    id,
                    KindScanVisibility::for_selected_version(
                        self.runtime.current_version_id(),
                        version_id,
                    ),
                )
                .map_err(request)?;
                super::super::super::read_work::record_memory::claim(memory, clone_bytes)
                    .map_err(request)?;
                let Some(record) =
                    self.authoritative_relation_record_for_id_at_version(edition, id, version_id)
                else {
                    super::super::super::read_work::record_memory::release(memory, clone_bytes)
                        .map_err(request)?;
                    continue;
                };
                let actual = super::super::super::read_footprint::relation(&record);
                super::super::super::read_work::record_memory::settle_clone(
                    memory,
                    clone_bytes,
                    actual,
                )
                .map_err(request)?;
                let clone_bytes = actual;
                if record.kind.kind_id != kind_id
                    || record.lifecycle != crate::storage::data::RecordLifecycleState::Live
                    || !direction.matches_endpoint(&record, entity_id)
                {
                    drop(record);
                    super::super::super::read_work::record_memory::release(memory, clone_bytes)
                        .map_err(request)?;
                    continue;
                }
                spend(
                    super::super::super::read_prices::RESERVE_RECORD,
                    examined,
                    records.len(),
                )?;
                super::super::super::read_work::record_memory::prepare_push(
                    memory,
                    &mut records,
                    0,
                )
                .map_err(request)?;
                records.push(record);
            }
            super::super::super::read_work::record_memory::release(memory, iterator_bytes)
                .map_err(request)?;
            sort_authoritative_relation_records(&mut records);
            Ok(BoundedAdjacencyTruthRead::new(records, examined))
        })();
        ledger.settle_read_round(edition, &self.runtime.services.instrumentation);
        answer
    }
}
