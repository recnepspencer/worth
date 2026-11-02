//! Admit cache-result backing before mutation and retain it with that entry.
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::storage::partition::HistoricalKindCustody;

pub(super) fn admit(
    prior: Option<&HistoricalKindCustody>,
    bytes: u64,
    reserve: &mut dyn FnMut(
        u64,
    )
        -> Result<Option<worth_execution::ExecutionMemoryReservation>, Cause>,
) -> Result<Option<HistoricalKindCustody>, Cause> {
    if let Some(prior) = prior {
        let mut ticket = prior.lock().map_err(|_| Cause::WorkerFailed)?;
        let total = ticket
            .bytes()
            .checked_add(bytes)
            .ok_or(Cause::ChargedBytesOverflow)?;
        ticket
            .grow_for_allocation(total)
            .map_err(|denial| match denial.classify() {
                worth_execution::ExecutionAllocationDenialClass::Request(cause) => cause.into(),
                worth_execution::ExecutionAllocationDenialClass::Physical(
                    worth_execution::ExecutionPhysicalAllocationDenialKind::Layout,
                ) => Cause::ChargedBytesOverflow,
                worth_execution::ExecutionAllocationDenialClass::Physical(_) => Cause::WorkerFailed,
            })?;
        return Ok(Some(prior.clone()));
    }
    let bytes = bytes
        .checked_add(HistoricalKindCustody::layout_bytes())
        .ok_or(Cause::ChargedBytesOverflow)?;
    // None is the SDK's explicit native-only accounting mode. It creates no
    // free replacement account. Accounted originals retain their own ticket.
    reserve(bytes).map(|ticket| {
        ticket.map(|ticket| HistoricalKindCustody::new(std::sync::Mutex::new(ticket)))
    })
}
