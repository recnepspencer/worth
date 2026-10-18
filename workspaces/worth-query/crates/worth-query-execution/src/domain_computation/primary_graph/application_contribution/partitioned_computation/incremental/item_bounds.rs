//! Full item/call capacity derivation at cold or explicit repartition boundaries.
use super::super::super::super::fact_verification_observation as observed;
use super::retained::RetainedCall;
use std::collections::BTreeMap;
use worth_execution::{ChargedBytes, PartitionItemId};
pub(super) fn items<Item: ChargedBytes>(items: &BTreeMap<PartitionItemId, Item>) -> Option<u64> {
    items.values().try_fold(0_u64, |sum, item| {
        observed::computation_retained_size();
        sum.checked_add(
            (size_of::<(PartitionItemId, Item)>() + size_of::<(PartitionItemId, [u8; 32])>())
                as u64,
        )?
        .checked_add(item.additional_charged_bytes())
    })
}
pub(super) fn calls(calls: &BTreeMap<PartitionItemId, RetainedCall>) -> Option<u64> {
    calls.values().try_fold(0_u64, |sum, call| {
        observed::computation_retained_size();
        sum.checked_add(size_of::<(PartitionItemId, RetainedCall)>() as u64)?
            .checked_add(call.charge.additional_bytes()?)
    })
}
