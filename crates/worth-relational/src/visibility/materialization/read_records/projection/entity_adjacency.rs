//! One entity's adjacency, read on the view's own branch root.
//!
//! The version-addressed reader answers from the runtime's shared partition
//! edition, so a branch reading at its own version can observe a sibling's
//! commit whose version is older. This read resolves the adjacency list and
//! every relation record against the branch root instead, while charging work
//! exactly as the version-addressed read does: one unit per examined relation
//! and one per admitted record.

use crate::identity::data::{EntityId, KindId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::overlay::PartitionAccess;
use crate::storage::partition::{AdjacencyDirection, AdjacencyKindBasis};

use super::read_record_identity_ordering::relation_identity_order_key;
use super::VisibilityProjectionView;
use crate::visibility::materialization::read_records::reader::adjacency_work_ledger::AdjacencyLeaseLedger;
use crate::visibility::materialization::read_records::reader::{
    AdjacencyTruthReadLimitExceeded, BoundedAdjacencyTruthRead,
};

impl VisibilityProjectionView {
    /// Reads the selected adjacency under the request, without a second ceiling.
    pub fn outgoing_relations_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        kind_id: KindId,
    ) -> Result<
        Vec<crate::storage::data::RelationReadRecord>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        super::super::read_work::metered_with_memory(
            execution,
            |context, memory| {
                self.entity_relations_of_kind(
                    entity_id,
                    kind_id,
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
                answer
                    .as_ref()
                    .map_or(0, super::super::read_footprint::relations)
            },
        )?
    }

    pub fn bounded_outgoing_relations_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        kind_id: KindId,
        maximum_work_units: Option<usize>,
    ) -> Result<
        Result<BoundedAdjacencyTruthRead, AdjacencyTruthReadLimitExceeded>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        self.bounded_entity_relations_of_kind(
            execution,
            entity_id,
            kind_id,
            AdjacencyDirection::Outgoing,
            maximum_work_units,
        )
    }

    pub fn bounded_incoming_relations_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        kind_id: KindId,
        maximum_work_units: Option<usize>,
    ) -> Result<
        Result<BoundedAdjacencyTruthRead, AdjacencyTruthReadLimitExceeded>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        self.bounded_entity_relations_of_kind(
            execution,
            entity_id,
            kind_id,
            AdjacencyDirection::Incoming,
            maximum_work_units,
        )
    }

    /// Read adjacency under the caller's optional projection work limit.
    /// Request stops are consulted before each examined or retained record.
    pub fn bounded_outgoing_relations_of_kind_admitted(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        kind_id: KindId,
        maximum_work_units: Option<usize>,
        mut admit_work: impl FnMut(u64) -> Result<(), crate::execution::RelationalExecutionDenialCause>,
    ) -> Result<
        Result<BoundedAdjacencyTruthRead, AdjacencyTruthReadLimitExceeded>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        self.admitted_entity_relations_of_kind(
            execution,
            entity_id,
            kind_id,
            AdjacencyDirection::Outgoing,
            maximum_work_units,
            &mut admit_work,
        )
    }

    /// Read adjacency under the caller's optional projection work limit.
    /// Request stops are consulted before each examined or retained record.
    pub fn bounded_incoming_relations_of_kind_admitted(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        kind_id: KindId,
        maximum_work_units: Option<usize>,
        mut admit_work: impl FnMut(u64) -> Result<(), crate::execution::RelationalExecutionDenialCause>,
    ) -> Result<
        Result<BoundedAdjacencyTruthRead, AdjacencyTruthReadLimitExceeded>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        self.admitted_entity_relations_of_kind(
            execution,
            entity_id,
            kind_id,
            AdjacencyDirection::Incoming,
            maximum_work_units,
            &mut admit_work,
        )
    }

    fn admitted_entity_relations_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        kind_id: KindId,
        direction: AdjacencyDirection,
        maximum_work_units: Option<usize>,
        admit_work: &mut dyn FnMut(
            u64,
        )
            -> Result<(), crate::execution::RelationalExecutionDenialCause>,
    ) -> Result<
        Result<BoundedAdjacencyTruthRead, AdjacencyTruthReadLimitExceeded>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        use super::super::native_scan_stop::NativeScanStop;
        crate::execution::check_request(execution)?;
        super::super::read_work::scan_with_memory(
            execution,
            |_context, memory| {
                self.entity_relations_of_kind(
                    entity_id,
                    kind_id,
                    direction,
                    &mut |units, examined, reserved| {
                        if maximum_work_units.is_some_and(|maximum| examined + reserved >= maximum)
                        {
                            return Err(NativeScanStop::Limit(
                                AdjacencyTruthReadLimitExceeded::new(examined, reserved),
                            ));
                        }
                        crate::execution::check_request(execution)
                            .map_err(NativeScanStop::Request)?;
                        admit_work(units).map_err(NativeScanStop::Request)
                    },
                    memory,
                    &NativeScanStop::Request,
                )
            },
            |read| read.owned_bytes(),
        )
    }

    fn bounded_entity_relations_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        kind_id: KindId,
        direction: AdjacencyDirection,
        maximum_work_units: Option<usize>,
    ) -> Result<
        Result<BoundedAdjacencyTruthRead, AdjacencyTruthReadLimitExceeded>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        super::super::read_work::scan_with_memory(
            execution,
            |context, memory| {
                self.entity_relations_of_kind(
                    entity_id,
                    kind_id,
                    direction,
                    &mut |units, examined, reserved| {
                        if maximum_work_units.is_some_and(|maximum| examined + reserved >= maximum)
                        {
                            return Err(super::super::native_scan_stop::NativeScanStop::Limit(
                                AdjacencyTruthReadLimitExceeded::new(examined, reserved),
                            ));
                        }
                        context
                            .checkpoint(units)
                            .map_err(super::super::native_scan_stop::NativeScanStop::request)
                    },
                    memory,
                    &super::super::native_scan_stop::NativeScanStop::Request,
                )
            },
            |read| read.owned_bytes(),
        )
    }

    fn entity_relations_of_kind<E>(
        &self,
        entity_id: EntityId,
        kind_id: KindId,
        direction: AdjacencyDirection,
        spend: &mut dyn FnMut(u64, usize, usize) -> Result<(), E>,
        memory: &mut worth_execution::ExecutionResultMemory<'_>,
        request: &dyn Fn(crate::execution::RelationalExecutionDenialCause) -> E,
    ) -> Result<BoundedAdjacencyTruthRead, E> {
        let Some(root) = self.basis.root() else {
            return Ok(BoundedAdjacencyTruthRead::new(Vec::new(), 0));
        };
        let basis = AdjacencyKindBasis::of_current_version(
            self.basis.root_version() == Some(self.version_id()),
        );
        let mut ledger = AdjacencyLeaseLedger::default();
        let ids = ledger.lease(
            root.get_partition(entity_id.partition_id)
                .and_then(|partition| direction.table(partition).get(entity_id.slot_index())),
            basis,
            kind_id,
        );
        let answer = (|| {
            let mut records = Vec::new();
            let mut examined = 0;
            let mut iterator_bytes = 0;
            let ids = ids.try_iter(|_navigation, bytes| {
                super::super::read_work::record_memory::claim(memory, bytes).map_err(request)?;
                iterator_bytes = bytes;
                Ok(())
            })?;
            for id in ids.copied() {
                spend(
                    super::super::read_prices::EXAMINE_SLOT,
                    examined,
                    records.len(),
                )?;
                examined += 1;
                let clone_bytes =
                    super::ProjectionRecordSource::candidate_relation_heap_for_id(self, id)
                        .map_err(request)?;
                super::super::read_work::record_memory::claim(memory, clone_bytes)
                    .map_err(request)?;
                let Some(record) = self.project_relation_record(id) else {
                    super::super::read_work::record_memory::release(memory, clone_bytes)
                        .map_err(request)?;
                    continue;
                };
                let actual = super::super::read_footprint::relation(&record);
                super::super::read_work::record_memory::settle_clone(memory, clone_bytes, actual)
                    .map_err(request)?;
                let clone_bytes = actual;
                if record.kind.kind_id != kind_id
                    || record.lifecycle != RecordLifecycleState::Live
                    || !direction.matches_endpoint(&record, entity_id)
                {
                    drop(record);
                    super::super::read_work::record_memory::release(memory, clone_bytes)
                        .map_err(request)?;
                    continue;
                }
                spend(
                    super::super::read_prices::RESERVE_RECORD,
                    examined,
                    records.len(),
                )?;
                super::super::read_work::record_memory::prepare_push(memory, &mut records, 0)
                    .map_err(request)?;
                records.push(record);
            }
            super::super::read_work::record_memory::release(memory, iterator_bytes)
                .map_err(request)?;
            records.sort_unstable_by_key(relation_identity_order_key);
            Ok(BoundedAdjacencyTruthRead::new(records, examined))
        })();
        ledger.settle_instrumentation(&self.instrumentation);
        answer
    }
}
