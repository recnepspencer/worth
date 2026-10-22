//! Full item/call capacity derivation at cold or explicit repartition boundaries.
use super::retained::RetainedCall;
use std::collections::BTreeMap;
use worth_execution::PartitionItemId;
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
