//! Original-request dependency traversal and provider-owned placement custody.
use super::*;
use crate::durability::medium::{
    RelationalDurableColdSource, RelationalDurableMemberCustody,
    RelationalDurableMemberLocator as Member,
};
use crate::storage::residency::RelationalColdRegion;
use crate::storage::substrate::RecordHistoryCustody;
use std::sync::Arc;
use worth_execution::ExecutionOrderedMap;

pub(in crate::storage::residency) fn capture(
    cold: &RelationalColdRegion,
    source: &Arc<dyn RelationalDurableColdSource>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Arc<RecordHistoryCustody>> {
    let mut pending = ExecutionOrderedMap::<Member, ()>::default();
    let mut seen = ExecutionOrderedMap::<Member, ()>::default();
    pending
        .insert_cloned_native_for_request(&cold.history, &(), *request)
        .map_err(ordered)?;
    loop {
        // Read-only directory navigation does not bill itself. Admit its
        // bounded comparisons before entry_at, removal, seen and bases lookups.
        work(request, 4 * 11 * usize::BITS as u64)?;
        let Some((&member, _)) = pending.entry_at(0) else {
            break;
        };
        pending
            .remove_for_request(&member, *request)
            .map_err(ordered)?;
        if seen.contains_key(&member) {
            continue;
        }
        seen.insert_cloned_native_for_request(&member, &(), *request)
            .map_err(ordered)?;
        for base in cold.history_members.bases(member) {
            work(request, 2 * 11 * usize::BITS as u64)?;
            if !seen.contains_key(base) && !pending.contains_key(base) {
                pending
                    .insert_cloned_native_for_request(base, &(), *request)
                    .map_err(ordered)?;
            }
        }
    }
    let _roster = Funding::reserve(array::<Member>(seen.len())?, request)?;
    let mut members = Vec::with_capacity(seen.len());
    for &member in seen.keys() {
        work(request, 1)?;
        members.push(member);
    }
    let memory =
        RecordHistoryCustody::reserve(request).map_err(|cause| Failure::Request(cause.into()))?;
    let token = RelationalDurableMemberCustody::capture(source, request, &members)
        .map_err(|cause| owned_read(cause, cold.partition_id))?;
    Ok(RecordHistoryCustody::new(source.clone(), token, memory))
}
