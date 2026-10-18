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
/// Cold or explicit full reconstruction of measured item calls. Ordinary
/// rekeys share this root and replace only actually executed calls.
pub(super) fn retain_calls(
    calls: &BTreeMap<PartitionItemId, RetainedCall>,
    policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    std::sync::Arc<worth_execution::ExecutionOrderedMap<PartitionItemId, RetainedCall>>,
    worth_execution::ExecutionOrderedMapDenial,
> {
    let mut retained = worth_execution::ExecutionOrderedMap::default();
    for (key, value) in calls {
        retained.insert_cloned_in(key, value, policy)?;
    }
    Ok(std::sync::Arc::new(retained))
}
