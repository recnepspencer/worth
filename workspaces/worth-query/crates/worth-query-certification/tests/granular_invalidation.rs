#[path = "../../worth-query-host/tests/temporal_conditional_operation/adapters.rs"]
mod adapters;
#[path = "../../worth-query-host/tests/temporal_conditional_operation/contract.rs"]
mod contract;
#[path = "../../worth-query-host/tests/temporal_conditional_operation/world.rs"]
mod host_world;
#[path = "../../worth-query-host/tests/temporal_conditional_operation/schema.rs"]
mod schema;
#[path = "support/execution_policy.rs"]
mod test_request_policy;

#[path = "granular_invalidation/delivery_convergence.rs"]
mod delivery_convergence;
#[path = "granular_invalidation/financial_runtime_world.rs"]
mod financial_runtime_world;
#[path = "granular_invalidation/heal_refresh.rs"]
mod heal_refresh;
#[path = "granular_invalidation/query_runtime_world.rs"]
mod query_runtime_world;
#[path = "granular_invalidation/runtime_composition.rs"]
mod runtime_composition;
#[path = "granular_invalidation/shared_lifecycle.rs"]
mod shared_lifecycle;
#[path = "granular_invalidation/structural_slopes.rs"]
mod structural_slopes;

#[test]
fn implemented_structural_slopes_run_through_the_real_composition_root() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    structural_slopes::assert_measured_bridge_and_result_slopes(request_execution);
}

#[test]
fn primary_runtime_carries_real_direct_truth_and_performed_signal_evidence() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    runtime_composition::assert_primary_runtime_composition(request_execution);
}

#[test]
fn duplicate_and_reordered_deliveries_converge_before_query_maintenance() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    delivery_convergence::assert_duplicate_and_reordered_convergence(request_execution);
}

#[test]
fn primary_runtime_rejects_a_foreign_primary_source_adapter() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    query_runtime_world::assert_foreign_primary_source_is_denied_at_build(request_execution);
}

#[test]
fn primary_runtime_retains_the_admitted_read_basis_across_a_head_advance() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    runtime_composition::assert_head_advance_preserves_admitted_granular_read(request_execution);
}

#[test]
fn primary_runtime_stamps_granular_receipts_from_the_execution_basis() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    runtime_composition::assert_granular_receipt_uses_execution_snapshot_basis(request_execution);
}

#[test]
fn healed_conditional_observation_refreshes_live_projection() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    heal_refresh::assert_healed_observation_refreshes_live_projection(request_execution);
}

#[test]
fn financial_curve_host_emits_real_granular_signal_delivery() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    financial_runtime_world::assert_financial_host_curve_delivery(request_execution);
}

#[test]
fn financial_curve_detail_performs_query_owned_risk_patch() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    financial_runtime_world::assert_financial_curve_query_patch(request_execution);
}

#[test]
fn financial_curve_detail_does_no_work_for_sibling_record_consumer() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    financial_runtime_world::assert_sibling_curve_record_does_no_query_work(request_execution);
}

#[test]
fn financial_quote_tolerance_suppresses_then_publishes_query_patch() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    financial_runtime_world::assert_suppressed_quote_has_no_query_patch(request_execution);
}

#[test]
fn ordered_portfolio_preserves_all_granular_query_consequences() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    financial_runtime_world::assert_ordered_portfolio_membership(request_execution);
}

#[test]
fn shared_primary_consumers_execute_once_and_publish_per_lease() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    financial_runtime_world::assert_shared_financial_execution_and_publication(request_execution);
}

#[test]
fn shared_primary_disclosure_is_revalidated_after_selection() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    financial_runtime_world::assert_shared_financial_disclosure_revalidation(request_execution);
}

#[test]
fn correspondence_restore_requires_current_rebinding_at_every_owner() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    shared_lifecycle::assert_correspondence_rebind_restore(request_execution);
}
