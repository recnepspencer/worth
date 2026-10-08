use super::{RetainedFact, StoreDenial};
use std::cell::RefCell;
use worth_execution::{ExecutionArray, ExecutionArrayBuilder};

/// Optional authored-order handoff, never a second authoritative lookup index.
/// Projected facts use canonical store output directly. Ordinary source arrays
/// already in authored order should bypass this bridge and retain that order.
pub(in crate::domain_computation::primary_graph) fn restore_authored_order<T>(
    records: ExecutionArray<RetainedFact<T>>,
    policy: super::StorageControl<'_, '_>,
) -> Result<ExecutionArray<RetainedFact<T>>, StoreDenial> {
    let count = records.len();
    policy.check_live()?;
    let mut slots = ExecutionArrayBuilder::allocate(count, policy.policy())?;
    for _ in 0..count {
        policy.check_live()?;
        slots.push(RefCell::new(None))?;
    }
    let slots = slots.seal()?;
    let mut records = records.into_iter();
    for record in records.by_ref() {
        policy.check_live()?;
        let slot = slots
            .get(record.ordinal)
            .ok_or(StoreDenial::InvalidOrdinal)?;
        let mut slot = slot.borrow_mut();
        if slot.is_some() {
            return Err(StoreDenial::InvalidOrdinal);
        }
        *slot = Some(record);
    }
    // Retain the original backing until all records have reached admitted slots.
    drop(records);
    policy.check_live()?;
    let mut output = ExecutionArrayBuilder::allocate(count, policy.policy())?;
    for slot in &slots {
        policy.check_live()?;
        output.push(
            slot.borrow_mut()
                .take()
                .ok_or(StoreDenial::InvalidOrdinal)?,
        )?;
    }
    Ok(output.seal()?)
}
