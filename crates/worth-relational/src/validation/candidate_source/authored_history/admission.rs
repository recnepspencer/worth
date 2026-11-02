//! Original Source-stage work and original-request storage, with owned stops.
use crate::derived_relation::{DerivedRelationDenial, DerivedRelationReadDenial};
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::validation::candidate_source::ManagedIndexSourcePreparationDenial as Denial;
use crate::validation::custom_rule::CustomPreparationBudget;
use worth_execution::{ExecutionOrderedMapDenial, ExecutionRequest};

pub(super) fn work(budget: &CustomPreparationBudget, units: usize) -> Result<(), Denial> {
    budget.try_visit(units).then_some(()).ok_or_else(|| {
        DerivedRelationReadDenial::Definition(DerivedRelationDenial::WorkBudgetExceeded).into()
    })
}
pub(super) fn ordered(
    budget: &CustomPreparationBudget,
    len: usize,
    width: usize,
) -> Result<(), Denial> {
    let height = 2 * (usize::BITS - len.leading_zeros()) as usize + 2;
    let units = height
        .checked_mul(width.max(1))
        .ok_or_else(|| storage(Cause::WorkCounterOverflow))?;
    work(budget, units)
}
pub(super) fn storage(cause: Cause) -> Denial {
    DerivedRelationReadDenial::Request(cause).into()
}
pub(super) fn map(cause: ExecutionOrderedMapDenial) -> Denial {
    use worth_execution::ExecutionAllocationDenialClass as Class;
    match cause {
        ExecutionOrderedMapDenial::Request(cause) => storage(cause.into()),
        ExecutionOrderedMapDenial::CapacityOverflow => storage(Cause::ChargedBytesOverflow),
        ExecutionOrderedMapDenial::Allocation(cause) => match cause.classify() {
            Class::Request(cause) => storage(cause.into()),
            Class::Physical(_) => storage(Cause::AllocationFailed),
        },
    }
}
pub(super) fn header<T>(
    execution: ExecutionRequest<'_, '_>,
    variable: usize,
) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Denial> {
    let bytes = std::alloc::Layout::new::<T>()
        .extend(std::alloc::Layout::new::<[usize; 2]>())
        .ok()
        .and_then(|(layout, _)| layout.pad_to_align().size().checked_add(variable))
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(|| storage(Cause::ChargedBytesOverflow))?;
    execution
        .reserve_retained_memory(bytes)
        .map_err(|cause| storage(cause.into()))
}
