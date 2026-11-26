use crate::internal_port::mvcc::observation::RelationalBranchObservationOwnerPort as _;
use std::sync::Arc;

use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::change_source::retained_observation::RelationalRetainedObservation;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::storage::data::RecordLifecycleState;
use crate::transactions::data::primitives::RecordRef;

use super::denial::RelationalRecordHistoryDenial as Denial;

/// Exact generation resolved in a retained native root, including a retained tombstone.
/// This provenance does not authorize application disclosure or product ancestry.
#[derive(Clone, Debug)]
pub struct RelationalRecordHistorySubject {
    owner: Arc<RetainedSubject>,
}

#[derive(Debug)]
struct RetainedSubject {
    record: RecordRef,
    retained: RelationalRetainedObservation,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl RelationalRecordHistorySubject {
    /// New fixed subject backing only; the admitted observation/root stays shared.
    pub const fn owned_backing_bytes() -> usize {
        size_of::<RetainedSubject>() + 2 * size_of::<usize>()
    }

    pub fn record(&self) -> &RecordRef {
        &self.owner.record
    }

    pub fn observation(&self) -> &crate::mvcc::observation::RelationalBranchObservation {
        self.owner.retained.observation()
    }

    pub(super) fn runtime_instance_id(&self) -> u64 {
        self.observation().identity().runtime_instance_id()
    }
}

pub fn require_exact_subject(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    symbols: &RuntimeSymbolTable,
    basis: &AdmittedRelationalBranchBasis,
    record: &RecordRef,
) -> Result<(), Denial> {
    let observation = basis.observation();
    let root = observation.selected_root().read();
    let partition_id = match record {
        RecordRef::Entity(id) => id.partition_id,
        RecordRef::Relation(id) => id.partition_id,
    };
    root.demand_regions_head(execution, symbols, [partition_id])
        .map_err(|cause| Denial::Storage(cause.into()))?;
    let resolved = match record {
        RecordRef::Entity(id) => {
            root.accepted_head_region_view(id.partition_id)
                .and_then(|partition| {
                    let slot = partition.entity_arena().get_slot(id.slot_index())?;
                    Some((
                        id.generation_value(),
                        slot.generation(),
                        slot.lifecycle(),
                        slot.kind_id(),
                    ))
                })
        }
        RecordRef::Relation(id) => {
            root.accepted_head_region_view(id.partition_id)
                .and_then(|partition| {
                    let slot = partition.relation_arena().get_slot(id.slot_index())?;
                    Some((
                        id.generation_value(),
                        slot.generation(),
                        slot.lifecycle(),
                        slot.kind_id(),
                    ))
                })
        }
    };
    let Some((requested, actual, lifecycle, kind)) = resolved else {
        return Err(Denial::SubjectUnavailable);
    };
    let retained_record = matches!(
        lifecycle,
        RecordLifecycleState::Live
            | RecordLifecycleState::MaterializationUnavailable
            | RecordLifecycleState::DeletedRetained
            | RecordLifecycleState::RetainedDanglingForAudit
            | RecordLifecycleState::PinnedBySnapshot
            | RecordLifecycleState::PinnedByBranch
            | RecordLifecycleState::PinnedByReplayRetention
    );
    if requested == 0 || requested != actual || kind.is_none() || !retained_record {
        return Err(Denial::SubjectUnavailable);
    }
    Ok(())
}

/// Operations used by the Relational owners.
pub trait RelationalRecordHistorySubjectOwnerPort: Sized {
    fn from_retained(
        record: RecordRef,
        retained: RelationalRetainedObservation,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Self;
}

impl RelationalRecordHistorySubjectOwnerPort for RelationalRecordHistorySubject {
    fn from_retained(
        record: RecordRef,
        retained: RelationalRetainedObservation,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            owner: Arc::new(RetainedSubject {
                record,
                retained,
                _memory: memory,
            }),
        }
    }
}
