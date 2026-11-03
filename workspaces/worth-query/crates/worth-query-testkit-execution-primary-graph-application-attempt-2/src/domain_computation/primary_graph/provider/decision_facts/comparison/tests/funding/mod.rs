//! An uncharged resident table still needs a genuine metered capture grant.
use crate::domain_computation::primary_graph::provider::decision_facts::comparison::tests::*;
use worth_execution::ExecutionAllocationDenialClass;
use worth_execution::LeaseDenial;

pub fn assert_full_capture_funding(
    provider: &WorthQueryPrimaryGraphProvider,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    authority: &WorthQuerySessionReadAuthority<'_>,
    request: &WorthQueryRequestScope,
    table: &worth_query_execution::domain_computation::primary_graph::provider::application_decision_fact::application_table::ApplicationFactTable,
) {
    // The actual registered fixture selected SystemAllocation. Its complete
    // original-fact path must retain the quote independently of that policy.
    assert!(table.closed_prefix().is_none());
    let bytes = table.full_capture_bytes().unwrap();
    assert!(bytes > 0);
    let tight = crate::test_request_policy::memory_child(*execution, bytes - 1);
    let refused = provider
        .capture_application_read_set(
            execution,
            authority,
            StorageControl::new(ExecutionAllocationPolicy::Execution(&tight), Some(request)),
        )
        .err()
        .expect("original-fact capture must admit its full copy quote first");
    let refusal = refused.allocation_denial().unwrap();
    assert_eq!(refusal.requested_payload_bytes(), Some(bytes));
    assert!(matches!(
        refusal.classify(),
        ExecutionAllocationDenialClass::Request(worth_execution::WorkCeilingDenial::Admission(
            LeaseDenial::MemoryExhausted(_)
        ))
    ));
    assert_eq!(tight.charged_memory_bytes(), 0);

    let funded = crate::test_request_policy::memory_child(*execution, 64 << 20);
    let receipt = provider
        .capture_application_read_set(
            execution,
            authority,
            StorageControl::new(ExecutionAllocationPolicy::Execution(&funded), Some(request)),
        )
        .unwrap();
    assert_eq!(receipt.fact_count(), table.len());
    assert!(funded.charged_memory_bytes() >= bytes);
    drop(receipt);
    assert_eq!(funded.charged_memory_bytes(), 0);
}
