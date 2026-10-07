#[path = "../../worth-query-host/tests/temporal_conditional_operation/adapters.rs"]
mod adapters;
#[path = "../../worth-query-host/tests/temporal_conditional_operation/contract.rs"]
mod contract;
#[path = "../../worth-query-host/tests/temporal_conditional_operation/world.rs"]
mod host_world;
#[path = "../../worth-query-host/tests/temporal_conditional_operation/schema.rs"]
mod schema;

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
    structural_slopes::assert_measured_bridge_and_result_slopes();
}

#[test]
fn primary_runtime_carries_real_direct_truth_and_performed_signal_evidence() {
    runtime_composition::assert_primary_runtime_composition();
}

#[test]
fn duplicate_and_reordered_deliveries_converge_before_query_maintenance() {
    delivery_convergence::assert_duplicate_and_reordered_convergence();
}

#[test]
fn primary_runtime_rejects_a_foreign_primary_source_adapter() {
    query_runtime_world::assert_foreign_primary_source_is_denied_at_build();
}

#[test]
fn primary_runtime_retains_the_admitted_read_basis_across_a_head_advance() {
    runtime_composition::assert_head_advance_preserves_admitted_granular_read();
}

#[test]
fn primary_runtime_stamps_granular_receipts_from_the_execution_basis() {
    runtime_composition::assert_granular_receipt_uses_execution_snapshot_basis();
}

#[test]
fn healed_conditional_observation_refreshes_live_projection() {
    heal_refresh::assert_healed_observation_refreshes_live_projection();
}

#[test]
fn financial_curve_host_emits_real_granular_signal_delivery() {
    financial_runtime_world::assert_financial_host_curve_delivery();
}

#[test]
fn financial_curve_detail_performs_query_owned_risk_patch() {
    financial_runtime_world::assert_financial_curve_query_patch();
}

#[test]
fn financial_curve_detail_does_no_work_for_sibling_record_consumer() {
    financial_runtime_world::assert_sibling_curve_record_does_no_query_work();
}

#[test]
fn financial_quote_tolerance_suppresses_then_publishes_query_patch() {
    financial_runtime_world::assert_suppressed_quote_has_no_query_patch();
}

#[test]
fn ordered_portfolio_preserves_all_granular_query_consequences() {
    financial_runtime_world::assert_ordered_portfolio_membership();
}

#[test]
fn shared_primary_consumers_execute_once_and_publish_per_lease() {
    financial_runtime_world::assert_shared_financial_execution_and_publication();
}

#[test]
fn shared_primary_disclosure_is_revalidated_after_selection() {
    financial_runtime_world::assert_shared_financial_disclosure_revalidation();
}

#[test]
fn correspondence_restore_requires_current_rebinding_at_every_owner() {
    shared_lifecycle::assert_correspondence_rebind_restore();
}
