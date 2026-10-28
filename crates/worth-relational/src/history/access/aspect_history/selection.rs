use std::sync::Arc;

use crate::branch::AdmittedRelationalBranchBasis;
use crate::change_source::RelationalRetainedObservation;
use crate::runtime::RelationalRuntime;
use crate::storage::data::RecordLifecycleState;
use crate::transactions::data::RecordRef;

use super::RelationalRecordHistoryDenial as Denial;

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

    pub fn observation(&self) -> &crate::mvcc::RelationalBranchObservation {
        self.owner.retained.observation()
    }

    pub(super) fn runtime_instance_id(&self) -> u64 {
        self.observation().identity().runtime_instance_id()
    }
}

impl RelationalRuntime {
    /// Resolve only the exact generation in this immutable root. An identifier
    /// alone cannot recreate the returned retained subject after reclamation/reuse.
    pub fn retain_record_history_subject(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        record: RecordRef,
    ) -> Result<RelationalRecordHistorySubject, Denial> {
        execution.check_live()?;
        if basis.identity().runtime_instance_id() != self.runtime_instance_id() {
            return Err(Denial::ForeignOwner);
        }
        require_exact_subject(&execution, self, basis, &record)?;
        let memory = execution
            .reserve_retained_memory(RelationalRecordHistorySubject::owned_backing_bytes() as u64)
            .map_err(Denial::Request)?;
        let retained = self
            .retain_observation_snapshot(basis)
            .map_err(Denial::Retention)?;
        execution.check_live()?;
        Ok(RelationalRecordHistorySubject {
            owner: Arc::new(RetainedSubject {
                record,
                retained,
                _memory: memory,
            }),
        })
    }
}

fn require_exact_subject(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    record: &RecordRef,
) -> Result<(), Denial> {
    let observation = basis.observation();
    let root = observation.selected_root();
    let partition_id = match record {
        RecordRef::Entity(id) => id.partition_id,
        RecordRef::Relation(id) => id.partition_id,
    };
    root.demand_regions_head(execution, &runtime.services.symbols, [partition_id])
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
