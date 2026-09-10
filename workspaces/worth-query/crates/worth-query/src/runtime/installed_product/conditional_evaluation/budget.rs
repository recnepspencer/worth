use super::entry::WorthQueryConditionalEvaluationEntry;
use super::retention::WorthQueryConditionalEvaluationRetentionLedger;

pub(super) fn retained_entry_allocation_charge() -> Result<u64, ()> {
    arc_allocation_charge::<WorthQueryConditionalEvaluationEntry>()?
        .checked_add(arc_allocation_charge::<
            worth_runtime_bridge::facade::BridgeConditionalEvaluationSession,
        >()?)
        .ok_or(())
}

pub(super) fn retention_ledger_allocation_charge() -> Result<u64, ()> {
    arc_allocation_charge::<WorthQueryConditionalEvaluationRetentionLedger>()
}

pub(super) fn vector_backing_charge<T>(capacity: usize) -> Result<u64, ()> {
    u64::try_from(std::mem::size_of::<T>())
        .ok()
        .and_then(|width| width.checked_mul(u64::try_from(capacity).ok()?))
        .ok_or(())
}

fn arc_allocation_charge<T>() -> Result<u64, ()> {
    let word = u64::try_from(std::mem::size_of::<usize>()).map_err(|_| ())?;
    let payload = u64::try_from(std::mem::size_of::<T>()).map_err(|_| ())?;
    let alignment = u64::try_from(std::mem::align_of::<T>().max(std::mem::align_of::<usize>()))
        .map_err(|_| ())?;
    word.checked_mul(2)
        .and_then(|header| header.checked_add(payload))
        .and_then(|charge| alignment.checked_mul(2)?.checked_add(charge))
        .ok_or(())
}
