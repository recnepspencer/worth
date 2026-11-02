//! Original-funded immutable Source condition and disposable entry headers.
use super::*;
use crate::mvcc::CompanionPreflightStop as Stop;
use worth_execution::ExecutionRequest;

pub(crate) struct ManagedIndexSourceAssociation {
    pub(crate) program: Program,
    pub(crate) scope: EntityId,
    pub(crate) availability: Availability,
    pub(crate) revision: crate::history::data::CommitId,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl ManagedIndexSourceAssociation {
    pub(super) fn new(
        execution: &ExecutionRequest<'_, '_>,
        program: &Program,
        scope: EntityId,
        availability: Availability,
        revision: crate::history::data::CommitId,
    ) -> Result<Arc<Self>, Stop> {
        let memory = header_memory::<Self>(execution)?;
        Ok(Arc::new(Self {
            program: program.clone(),
            scope,
            availability,
            revision,
            _memory: memory,
        }))
    }
}
impl ManagedIndexPublishedSourceRoot {
    pub(super) fn new(
        execution: &ExecutionRequest<'_, '_>,
        association: Arc<ManagedIndexSourceAssociation>,
        payload: Option<Payload>,
    ) -> Result<Arc<Self>, Stop> {
        let memory = header_memory::<Self>(execution)?;
        Ok(Arc::new(Self {
            association,
            payload,
            _memory: memory,
        }))
    }
}
fn header_memory<T>(
    execution: &ExecutionRequest<'_, '_>,
) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Stop> {
    let bytes = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .ok()
        .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
        .ok_or(Stop::PreparationMemoryCounterOverflow)?;
    execution
        .reserve_retained_memory(bytes)
        .map_err(|cause| Stop::Execution(cause.into()))
}
pub(super) fn ordered_denial(cause: worth_execution::ExecutionOrderedMapDenial) -> Stop {
    use crate::execution::RelationalExecutionDenialCause as Cause;
    use worth_execution::{
        ExecutionAllocationDenialClass as Class, ExecutionOrderedMapDenial as Denial,
    };
    match cause {
        Denial::Request(cause) => Stop::Execution(cause.into()),
        Denial::CapacityOverflow => Stop::PreparationMemoryCounterOverflow,
        Denial::Allocation(cause) => match cause.classify() {
            Class::Request(cause) => Stop::Execution(cause.into()),
            Class::Physical(_) => Stop::Execution(Cause::AllocationFailed),
        },
    }
}
