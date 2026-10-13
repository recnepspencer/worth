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
        super::super::read_work::metered(
            execution,
            |context| {
                self.entity_relations_of_kind(
                    entity_id,
                    kind_id,
                    AdjacencyDirection::Outgoing,
                    &mut |units, _, _| context.checkpoint(units),
                )
                .map(BoundedAdjacencyTruthRead::into_records)
            },
            |answer| {
                answer
                    .as_ref()
                    .map_or(0, super::super::read_footprint::relations)
            },
        )?
        .map_err(crate::execution::denial::kernel_stop)
    }

    pub fn bounded_outgoing_relations_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        kind_id: KindId,
        maximum_work_units: usize,
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
        maximum_work_units: usize,
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

    /// Read bounded adjacency already admitted by a caller's projection budget.
    /// Request stops are consulted before each examined or retained record.
    pub fn bounded_outgoing_relations_of_kind_admitted(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        kind_id: KindId,
        maximum_work_units: usize,
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

    /// Read bounded adjacency already admitted by a caller's projection budget.
    /// Request stops are consulted before each examined or retained record.
    pub fn bounded_incoming_relations_of_kind_admitted(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        kind_id: KindId,
        maximum_work_units: usize,
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
        maximum_work_units: usize,
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
        let read = self.entity_relations_of_kind(
            entity_id,
            kind_id,
            direction,
            &mut |units, examined, reserved| {
                if examined + reserved == maximum_work_units {
                    return Err(NativeScanStop::Limit(AdjacencyTruthReadLimitExceeded::new(
                        examined, reserved,
                    )));
                }
                crate::execution::check_request(execution).map_err(NativeScanStop::Request)?;
                admit_work(units).map_err(NativeScanStop::Request)
            },
        );
        match read {
            Ok(read) => Ok(Ok(read)),
            Err(NativeScanStop::Limit(limit)) => Ok(Err(limit)),
            Err(NativeScanStop::Request(cause)) => Err(cause),
        }
    }

    fn bounded_entity_relations_of_kind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: EntityId,
        kind_id: KindId,
        direction: AdjacencyDirection,
        maximum_work_units: usize,
    ) -> Result<
        Result<BoundedAdjacencyTruthRead, AdjacencyTruthReadLimitExceeded>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        super::super::read_work::scan(
            execution,
            |context| {
                self.entity_relations_of_kind(
                    entity_id,
                    kind_id,
                    direction,
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
            |read| read.owned_bytes(),
        )
    }

    fn entity_relations_of_kind<E>(
        &self,
        entity_id: EntityId,
        kind_id: KindId,
        direction: AdjacencyDirection,
        spend: &mut dyn FnMut(u64, usize, usize) -> Result<(), E>,
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
            for id in ids.iter().copied() {
                spend(
                    super::super::read_prices::EXAMINE_SLOT,
                    examined,
                    records.len(),
                )?;
                examined += 1;
                let Some(record) = self.project_relation_record(id) else {
                    continue;
                };
                if record.kind.kind_id != kind_id
                    || record.lifecycle != RecordLifecycleState::Live
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
            records.sort_by_key(relation_identity_order_key);
            Ok(BoundedAdjacencyTruthRead::new(records, examined))
        })();
        ledger.settle_instrumentation(&self.instrumentation);
        answer
    }
}
