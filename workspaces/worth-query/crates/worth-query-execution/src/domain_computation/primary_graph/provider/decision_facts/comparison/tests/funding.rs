//! An uncharged resident table still needs a genuine metered capture grant.
use super::*;
use worth_execution::{ExecutionAllocationDenialKind, ExecutionResourceLease, LeaseDenial};

pub(super) fn assert_full_capture_funding(
    provider: &WorthQueryPrimaryGraphProvider,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    authority: &WorthQuerySessionReadAuthority<'_>,
    request: &WorthQueryRequestScope,
    table: &crate::domain_computation::primary_graph::ApplicationFactTable,
    caller: &ExecutionResourceLease<'_>,
) {
    // The actual registered fixture selected SystemAllocation. Its complete
    // original-fact path must retain the quote independently of that policy.
    assert!(table.closed_prefix().is_none());
    let bytes = table.full_capture_bytes().unwrap();
    assert!(bytes > 0);
    // `caller` funds the open call; leave it one byte short of the quote.
    let room = caller.available_memory_bytes().unwrap();
    let tight = caller.reserve_memory(room - (bytes - 1)).unwrap();
    let working = caller.charged_memory_bytes();
    let refused = provider
        .capture_application_read_set(
            execution,
            authority,
            StorageControl::new(ExecutionAllocationPolicy::Execution(caller), Some(request)),
        )
        .err()
        .expect("original-fact capture must admit its full copy quote first");
    let refusal = refused.allocation_denial().unwrap();
    assert_eq!(refusal.requested_payload_bytes(), Some(bytes));
    assert!(matches!(
        refusal.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(_))
    ));
    assert_eq!(caller.charged_memory_bytes(), working);
    drop(tight);

    let working = caller.charged_memory_bytes();
    let receipt = provider
        .capture_application_read_set(
            execution,
            authority,
            StorageControl::new(ExecutionAllocationPolicy::Execution(caller), Some(request)),
        )
        .unwrap();
    assert_eq!(receipt.fact_count(), table.len());
    assert!(caller.charged_memory_bytes() >= working + bytes);
    drop(receipt);
    assert_eq!(caller.charged_memory_bytes(), working);
}
